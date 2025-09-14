use eframe::egui::{self, Ui, Pos2, Rect, Color32, Stroke, FontId, Vec2};
use std::sync::{Arc, Mutex};
use crate::backend::{BackendState, WorkflowGraph, GraphNode, greet, get_workflow_graph, list_plugins_for_ui, run_workflow_stream, save_workflow_yaml, export_workflow_yaml};

pub struct LaoApp {
    state: Arc<Mutex<BackendState>>,
    
    // UI state
    name_input: String,
    greet_msg: String,
    new_node_name: String,
    new_node_type: String,
    new_workflow_filename: String,
    
    // Visual editor state
    dragging_node: Option<String>,
    drag_offset: Vec2,
    connecting_from: Option<String>,
    show_save_dialog: bool,
    show_export_dialog: bool,
}

impl LaoApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut state = BackendState::default();
        
        // Try to load plugins on startup
        if let Ok(plugins) = list_plugins_for_ui() {
            state.plugins = plugins;
        }
        
        Self {
            state: Arc::new(Mutex::new(state)),
            name_input: String::new(),
            greet_msg: String::new(),
            new_node_name: String::new(),
            new_node_type: "Echo".to_string(),
            new_workflow_filename: "new_workflow.yaml".to_string(),
            dragging_node: None,
            drag_offset: Vec2::ZERO,
            connecting_from: None,
            show_save_dialog: false,
            show_export_dialog: false,
        }
    }
}

impl eframe::App for LaoApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("LAO Orchestrator");
            
            ui.separator();
            
            // Greeting section
            self.show_greeting_section(ui);
            
            ui.separator();
            
            // Workflow section
            self.show_workflow_section(ui);
            
            ui.separator();
            
            // Visual graph editor
            self.show_visual_editor(ui);
            
            ui.separator();
            
            // Live logs section
            self.show_live_logs_section(ui);
        });
    }
}

impl LaoApp {
    fn show_greeting_section(&mut self, ui: &mut Ui) {
        ui.heading("Greeting");
        
        ui.horizontal(|ui| {
            ui.label("Your name:");
            ui.text_edit_singleline(&mut self.name_input);
            
            if ui.button("Greet").clicked() {
                self.greet_msg = greet(&self.name_input);
            }
        });
        
        if !self.greet_msg.is_empty() {
            ui.label(&self.greet_msg);
        }
    }
    
    fn show_workflow_section(&mut self, ui: &mut Ui) {
        ui.heading("Workflow Management");
        
        let mut state = self.state.lock().unwrap();
        
        ui.horizontal(|ui| {
            ui.label("Workflow path:");
            ui.text_edit_singleline(&mut state.workflow_path);
            
            if ui.button("Load Workflow").clicked() {
                match get_workflow_graph(&state.workflow_path) {
                    Ok(graph) => {
                        state.graph = Some(graph);
                        state.error.clear();
                    }
                    Err(e) => {
                        state.error = e;
                        state.graph = None;
                    }
                }
            }
            
            if ui.button("Run").clicked() {
                if !state.workflow_path.is_empty() && !state.is_running {
                    if let Some(ref graph) = state.graph {
                        // Reset node statuses before execution
                        let mut graph_clone = graph.clone();
                        for node in &mut graph_clone.nodes {
                            node.status = "pending".to_string();
                            node.message = None;
                            node.output = None;
                            node.error = None;
                            node.attempt = 0;
                        }
                        state.graph = Some(graph_clone);
                    }
                    
                    let path = state.workflow_path.clone();
                    let state_ref = Arc::clone(&self.state);
                    let _ = run_workflow_stream(path, false, state_ref);
                }
            }
            
            if ui.button("Run (Parallel)").clicked() {
                if !state.workflow_path.is_empty() && !state.is_running {
                    if let Some(ref graph) = state.graph {
                        // Reset node statuses before execution
                        let mut graph_clone = graph.clone();
                        for node in &mut graph_clone.nodes {
                            node.status = "pending".to_string();
                            node.message = None;
                            node.output = None;
                            node.error = None;
                            node.attempt = 0;
                        }
                        state.graph = Some(graph_clone);
                    }
                    
                    let path = state.workflow_path.clone();
                    let state_ref = Arc::clone(&self.state);
                    let _ = run_workflow_stream(path, true, state_ref);
                }
            }
        });
        
        if !state.error.is_empty() {
            ui.colored_label(Color32::RED, &state.error);
        }
        
        // Show execution status
        if state.is_running {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label(format!("Executing workflow... {:.0}%", state.execution_progress * 100.0));
            });
            ui.add(egui::ProgressBar::new(state.execution_progress).show_percentage());
        }
        
        // Show execution results
        if let Some(ref result) = state.workflow_result {
            ui.horizontal(|ui| {
                if result.success {
                    ui.colored_label(Color32::GREEN, "✓");
                } else {
                    ui.colored_label(Color32::RED, "✗");
                }
                ui.label(&result.final_message);
            });
            
            ui.collapsing("Execution Summary", |ui| {
                ui.label(format!("Total steps: {}", result.total_steps));
                ui.label(format!("Completed: {}", result.completed_steps));
                ui.label(format!("Failed: {}", result.failed_steps));
                ui.label(format!("Execution time: {:.2}s", result.execution_time));
            });
        }
        
        // Show current graph info
        if let Some(ref graph) = state.graph {
            ui.collapsing("Graph Details", |ui| {
                ui.label(format!("Nodes: {}", graph.nodes.len()));
                ui.label(format!("Edges: {}", graph.edges.len()));
                
                for node in &graph.nodes {
                    ui.label(format!("• {}: {} ({})", node.id, node.run, node.status));
                }
                
                if !graph.edges.is_empty() {
                    ui.label("Edges:");
                    for edge in &graph.edges {
                        ui.label(format!("  {} → {}", edge.from, edge.to));
                    }
                }
            });
        }
    }
    
    fn show_visual_editor(&mut self, ui: &mut Ui) {
        ui.heading("Visual Flow Builder");
        
        ui.horizontal(|ui| {
            if ui.button("New Workflow").clicked() {
                let mut state = self.state.lock().unwrap();
                state.graph = Some(WorkflowGraph {
                    nodes: Vec::new(),
                    edges: Vec::new(),
                });
            }
            
            if ui.button("Save Workflow").clicked() {
                self.show_save_dialog = true;
            }
            
            if ui.button("Export YAML").clicked() {
                self.show_export_dialog = true;
            }
            
            // Show connection mode
            if self.connecting_from.is_some() {
                ui.colored_label(Color32::YELLOW, "🔗 Connection mode: Click target node");
                if ui.button("Cancel").clicked() {
                    self.connecting_from = None;
                }
            } else {
                ui.colored_label(Color32::GRAY, "💡 Tip: Right-click nodes for options, drag to move");
            }
        });
        
        // Save dialog
        if self.show_save_dialog {
            let mut close_dialog = false;
            egui::Window::new("Save Workflow")
                .open(&mut self.show_save_dialog)
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Filename:");
                        ui.text_edit_singleline(&mut self.new_workflow_filename);
                    });
                    
                    ui.horizontal(|ui| {
                        if ui.button("Save").clicked() {
                            let state = self.state.lock().unwrap();
                            if let Some(ref graph) = state.graph {
                                match save_workflow_yaml(graph, &self.new_workflow_filename) {
                                    Ok(_) => {
                                        close_dialog = true;
                                        // Could add success message
                                    }
                                    Err(e) => {
                                        // Could add error handling
                                        eprintln!("Save error: {}", e);
                                    }
                                }
                            }
                        }
                        
                        if ui.button("Cancel").clicked() {
                            close_dialog = true;
                        }
                    });
                });
            
            if close_dialog {
                self.show_save_dialog = false;
            }
        }
        
        // Export dialog  
        if self.show_export_dialog {
            let mut close_dialog = false;
            egui::Window::new("Export YAML")
                .open(&mut self.show_export_dialog)
                .show(ui.ctx(), |ui| {
                    let state = self.state.lock().unwrap();
                    if let Some(ref graph) = state.graph {
                        match export_workflow_yaml(graph) {
                            Ok(yaml) => {
                                ui.label("Generated YAML:");
                                egui::ScrollArea::vertical()
                                    .max_height(300.0)
                                    .show(ui, |ui| {
                                        ui.text_edit_multiline(&mut yaml.as_str());
                                    });
                            }
                            Err(e) => {
                                ui.colored_label(Color32::RED, format!("Export error: {}", e));
                            }
                        }
                    }
                    
                    if ui.button("Close").clicked() {
                        close_dialog = true;
                    }
                });
            
            if close_dialog {
                self.show_export_dialog = false;
            }
        }
        
        // Get the plugins list and selected node first
        let (plugins, selected_node_id) = {
            let state = self.state.lock().unwrap();
            (state.plugins.clone(), state.selected_node.clone())
        };
        
        // Handle interaction separately
        let mut node_clicked = None;
        let mut should_remove_node = false;
        
        {
            let mut state = self.state.lock().unwrap();
            
            if let Some(ref mut graph) = state.graph {
                // Add node controls
                ui.horizontal(|ui| {
                    ui.label("Add Node:");
                    ui.text_edit_singleline(&mut self.new_node_name);
                    
                    egui::ComboBox::from_label("Type")
                        .selected_text(&self.new_node_type)
                        .show_ui(ui, |ui| {
                            for plugin in &plugins {
                                ui.selectable_value(&mut self.new_node_type, plugin.name.clone(), &plugin.name);
                            }
                        });
                    
                    if ui.button("Add Node").clicked() {
                        let node_id = if self.new_node_name.is_empty() {
                            format!("node_{}", graph.nodes.len() + 1)
                        } else {
                            self.new_node_name.clone()
                        };
                        
                        graph.nodes.push(GraphNode {
                            id: node_id,
                            run: self.new_node_type.clone(),
                            input_type: None,
                            output_type: None,
                            status: "pending".to_string(),
                            x: 100.0 + (graph.nodes.len() as f32 * 150.0),
                            y: 100.0,
                            message: None,
                            output: None,
                            error: None,
                            attempt: 0,
                        });
                        
                        self.new_node_name.clear();
                    }
                });
                
                // Visual graph area
                let available_rect = ui.available_rect_before_wrap();
                let graph_rect = Rect::from_min_size(available_rect.min, egui::vec2(800.0, 400.0));
                
                let response = ui.allocate_rect(graph_rect, egui::Sense::click_and_drag());
                
                if ui.is_rect_visible(graph_rect) {
                    let painter = ui.painter();
                    
                    // Draw background
                    painter.rect_filled(graph_rect, 4.0, Color32::from_gray(248));
                    
                    // Draw grid
                    let grid_size = 40.0;
                    for i in 0..20 {
                        let x = graph_rect.min.x + i as f32 * grid_size;
                        if x < graph_rect.max.x {
                            painter.line_segment(
                                [Pos2::new(x, graph_rect.min.y), Pos2::new(x, graph_rect.max.y)],
                                Stroke::new(1.0, Color32::from_gray(238))
                            );
                        }
                    }
                    for j in 0..10 {
                        let y = graph_rect.min.y + j as f32 * grid_size;
                        if y < graph_rect.max.y {
                            painter.line_segment(
                                [Pos2::new(graph_rect.min.x, y), Pos2::new(graph_rect.max.x, y)],
                                Stroke::new(1.0, Color32::from_gray(238))
                            );
                        }
                    }
                    
                    // Draw edges with improved visualization
                    for (i, edge) in graph.edges.iter().enumerate() {
                        if let (Some(from_node), Some(to_node)) = (
                            graph.nodes.iter().find(|n| n.id == edge.from),
                            graph.nodes.iter().find(|n| n.id == edge.to)
                        ) {
                            let from_pos = Pos2::new(
                                graph_rect.min.x + from_node.x + 120.0,
                                graph_rect.min.y + from_node.y + 30.0
                            );
                            let to_pos = Pos2::new(
                                graph_rect.min.x + to_node.x,
                                graph_rect.min.y + to_node.y + 30.0
                            );
                            
                            // Draw arrow line
                            painter.line_segment(
                                [from_pos, to_pos],
                                Stroke::new(2.0, Color32::from_gray(136))
                            );
                            
                            // Draw arrowhead
                            let direction = (to_pos - from_pos).normalized();
                            let arrow_size = 8.0;
                            let arrow_tip = to_pos - direction * 5.0;
                            let perpendicular = Vec2::new(-direction.y, direction.x);
                            
                            let arrow_p1 = arrow_tip - direction * arrow_size + perpendicular * arrow_size * 0.5;
                            let arrow_p2 = arrow_tip - direction * arrow_size - perpendicular * arrow_size * 0.5;
                            
                            painter.line_segment([arrow_tip, arrow_p1], Stroke::new(2.0, Color32::from_gray(136)));
                            painter.line_segment([arrow_tip, arrow_p2], Stroke::new(2.0, Color32::from_gray(136)));
                            
                            // Check for edge click to delete
                            let edge_center = (from_pos + to_pos.to_vec2()) * 0.5;
                            let edge_rect = Rect::from_center_size(edge_center, Vec2::splat(20.0));
                            let edge_response = ui.interact(edge_rect, egui::Id::new(format!("edge_{}", i)), egui::Sense::click());
                            
                            if edge_response.secondary_clicked() {
                                // Mark edge for deletion (we'll handle this outside the loop)
                                // For now, we could add a context menu here
                            }
                        }
                    }
                    
                    // Draw nodes and handle interactions
                    for node in &mut graph.nodes {
                        let node_pos = Pos2::new(
                            graph_rect.min.x + node.x,
                            graph_rect.min.y + node.y
                        );
                        let node_rect = Rect::from_min_size(node_pos, egui::vec2(120.0, 60.0));
                        
                        // Node background color based on status
                        let node_color = match node.status.as_str() {
                            "running" => Color32::from_rgb(33, 150, 243),   // Blue
                            "success" => Color32::from_rgb(76, 175, 80),    // Green  
                            "error" => Color32::from_rgb(244, 67, 54),      // Red
                            "cache" => Color32::from_rgb(156, 39, 176),     // Purple
                            "pending" => Color32::from_rgb(96, 125, 139),   // Blue Gray
                            _ => Color32::from_rgb(34, 34, 34),             // Dark Gray
                        };
                        
                        painter.rect_filled(node_rect, 12.0, node_color);
                        
                        // Highlight node if it's the connection source
                        if self.connecting_from.as_ref() == Some(&node.id) {
                            painter.rect_stroke(node_rect, 12.0, Stroke::new(3.0, Color32::YELLOW));
                        } else {
                            painter.rect_stroke(node_rect, 12.0, Stroke::new(2.0, Color32::from_gray(68)));
                        }
                        
                        // Node text
                        painter.text(
                            node_rect.center() - egui::vec2(0.0, 8.0),
                            egui::Align2::CENTER_CENTER,
                            &node.id,
                            FontId::default(),
                            Color32::WHITE
                        );
                        
                        painter.text(
                            node_rect.center() + egui::vec2(0.0, 8.0),
                            egui::Align2::CENTER_CENTER,
                            format!("{} ({})", node.run, node.status),
                            FontId::proportional(10.0),
                            Color32::from_gray(221)
                        );
                        
                        // Handle node interaction
                        let node_response = ui.interact(node_rect, egui::Id::new(&node.id), egui::Sense::click_and_drag());
                        
                        if node_response.clicked() {
                            // Handle connection mode
                            if let Some(ref from_id) = self.connecting_from {
                                if from_id != &node.id {
                                    // Create edge
                                    let edge = crate::backend::GraphEdge {
                                        from: from_id.clone(),
                                        to: node.id.clone(),
                                    };
                                    if !graph.edges.iter().any(|e| e.from == edge.from && e.to == edge.to) {
                                        graph.edges.push(edge);
                                    }
                                }
                                self.connecting_from = None;
                            } else {
                                node_clicked = Some(node.id.clone());
                            }
                        }
                        
                        // Right-click for context menu
                        if node_response.secondary_clicked() {
                            node_clicked = Some(node.id.clone());
                            // In a real implementation, you'd show a context menu here
                        }
                        
                        if node_response.dragged() && self.connecting_from.is_none() {
                            if let Some(pointer_pos) = response.interact_pointer_pos() {
                                if self.dragging_node.is_none() {
                                    self.dragging_node = Some(node.id.clone());
                                    self.drag_offset = pointer_pos - node_pos;
                                }
                                
                                if self.dragging_node.as_ref() == Some(&node.id) {
                                    let new_pos = pointer_pos - self.drag_offset - graph_rect.min.to_vec2();
                                    let grid_size = 40.0;
                                    node.x = (new_pos.x / grid_size).round() * grid_size;
                                    node.y = (new_pos.y / grid_size).round() * grid_size;
                                }
                            }
                        }
                    }
                    
                    if response.drag_stopped() {
                        self.dragging_node = None;
                    }
                }
                
                // Node inspector
                if let Some(ref selected_id) = selected_node_id {
                    if let Some(selected_node) = graph.nodes.iter_mut().find(|n| n.id == *selected_id) {
                        ui.separator();
                        ui.heading("Node Inspector");
                        
                        ui.horizontal(|ui| {
                            ui.label("ID:");
                            ui.label(&selected_node.id);
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Run:");
                            egui::ComboBox::from_label("")
                                .selected_text(&selected_node.run)
                                .show_ui(ui, |ui| {
                                    for plugin in &plugins {
                                        ui.selectable_value(&mut selected_node.run, plugin.name.clone(), &plugin.name);
                                    }
                                });
                        });
                        
                        ui.horizontal(|ui| {
                            ui.label("Status:");
                            let status_color = match selected_node.status.as_str() {
                                "running" => Color32::BLUE,
                                "success" => Color32::GREEN,
                                "error" => Color32::RED,
                                "cache" => Color32::BROWN,
                                _ => Color32::GRAY,
                            };
                            ui.colored_label(status_color, &selected_node.status);
                        });
                        
                        if let Some(ref msg) = selected_node.message {
                            ui.horizontal(|ui| {
                                ui.label("Message:");
                                ui.label(msg);
                            });
                        }
                        
                        if let Some(ref output) = selected_node.output {
                            ui.collapsing("Output", |ui| {
                                egui::ScrollArea::vertical()
                                    .max_height(100.0)
                                    .show(ui, |ui| {
                                        ui.text_edit_multiline(&mut output.as_str());
                                    });
                            });
                        }
                        
                        if let Some(ref error) = selected_node.error {
                            ui.collapsing("Error", |ui| {
                                ui.colored_label(Color32::RED, error);
                            });
                        }
                        
                        ui.horizontal(|ui| {
                            if ui.button("Connect From").clicked() {
                                self.connecting_from = Some(selected_node.id.clone());
                            }
                            
                            if ui.button("Remove Node").clicked() {
                                should_remove_node = true;
                            }
                        });
                    }
                }
            }
        }
        
        // Handle state updates after dropping the lock
        {
            let mut state = self.state.lock().unwrap();
            if let Some(clicked_id) = node_clicked {
                state.selected_node = Some(clicked_id);
            }
            
            if should_remove_node {
                if let (Some(ref mut graph), Some(ref selected_id)) = (&mut state.graph, &selected_node_id) {
                    graph.nodes.retain(|n| n.id != *selected_id);
                    graph.edges.retain(|e| e.from != *selected_id && e.to != *selected_id);
                    state.selected_node = None;
                }
            }
        }
    }
    
    fn show_live_logs_section(&mut self, ui: &mut Ui) {
        ui.heading("Live Logs & Execution Status");
        
        let (is_running, execution_progress, workflow_result, logs) = {
            let state = self.state.lock().unwrap();
            (
                state.is_running,
                state.execution_progress,
                state.workflow_result.clone(),
                state.live_logs.clone()
            )
        };
        
        // Show execution status indicator
        ui.horizontal(|ui| {
            if is_running {
                ui.spinner();
                ui.colored_label(Color32::BLUE, "Workflow Executing");
                ui.add(egui::ProgressBar::new(execution_progress).show_percentage());
            } else if let Some(ref result) = workflow_result {
                if result.success {
                    ui.colored_label(Color32::GREEN, "✓ Execution Complete");
                } else {
                    ui.colored_label(Color32::RED, "✗ Execution Failed");
                }
            } else {
                ui.colored_label(Color32::GRAY, "Ready");
            }
        });
        
        ui.separator();
        
        // Log controls
        ui.horizontal(|ui| {
            ui.label("Logs:");
            if ui.button("Clear").clicked() {
                let mut state = self.state.lock().unwrap();
                state.live_logs.clear();
            }
        });
        
        // Live logs display
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .auto_shrink([false, true])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for log in &logs {
                    // Color code based on log content
                    let color = if log.contains("✓ DONE") {
                        Color32::GREEN
                    } else if log.contains("✗ ERROR") {
                        Color32::RED
                    } else if log.contains("running") {
                        Color32::BLUE
                    } else if log.contains("success") || log.contains("cache") {
                        Color32::GREEN
                    } else if log.contains("error") || log.contains("failed") {
                        Color32::RED
                    } else {
                        Color32::WHITE
                    };
                    
                    ui.colored_label(color, log);
                }
                
                // Show empty state
                if logs.is_empty() {
                    ui.colored_label(Color32::GRAY, "No logs yet. Run a workflow to see execution logs here.");
                }
            });
    }
}
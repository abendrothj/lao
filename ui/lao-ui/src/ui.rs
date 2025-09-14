use eframe::egui::{self, Ui, Pos2, Rect, Color32, Stroke, FontId, Vec2};
use std::sync::{Arc, Mutex};
use crate::backend::{BackendState, WorkflowGraph, GraphNode, greet, get_workflow_graph, list_plugins_for_ui, run_workflow_stream};

pub struct LaoApp {
    state: Arc<Mutex<BackendState>>,
    
    // UI state
    name_input: String,
    greet_msg: String,
    new_node_name: String,
    new_node_type: String,
    new_workflow_filename: String,
    yaml_export: String,
    prompt_input: String,
    generated_yaml: String,
    
    // Visual editor state
    dragging_node: Option<String>,
    drag_offset: Vec2,
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
            yaml_export: String::new(),
            prompt_input: String::new(),
            generated_yaml: String::new(),
            dragging_node: None,
            drag_offset: Vec2::ZERO,
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
                if !state.workflow_path.is_empty() {
                    let path = state.workflow_path.clone();
                    let logs = Arc::clone(&self.state);
                    let _ = run_workflow_stream(path, false, move |log| {
                        if let Ok(mut state) = logs.lock() {
                            state.live_logs.push(log);
                            if state.live_logs.len() > 200 {
                                state.live_logs.remove(0);
                            }
                        }
                    });
                }
            }
            
            if ui.button("Run (Parallel)").clicked() {
                if !state.workflow_path.is_empty() {
                    let path = state.workflow_path.clone();
                    let logs = Arc::clone(&self.state);
                    let _ = run_workflow_stream(path, true, move |log| {
                        if let Ok(mut state) = logs.lock() {
                            state.live_logs.push(log);
                            if state.live_logs.len() > 200 {
                                state.live_logs.remove(0);
                            }
                        }
                    });
                }
            }
        });
        
        if !state.error.is_empty() {
            ui.colored_label(Color32::RED, &state.error);
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
        
        if ui.button("New Workflow").clicked() {
            let mut state = self.state.lock().unwrap();
            state.graph = Some(WorkflowGraph {
                nodes: Vec::new(),
                edges: Vec::new(),
            });
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
                    
                    // Draw edges
                    for edge in &graph.edges {
                        if let (Some(from_node), Some(to_node)) = (
                            graph.nodes.iter().find(|n| n.id == edge.from),
                            graph.nodes.iter().find(|n| n.id == edge.to)
                        ) {
                            let from_pos = Pos2::new(
                                graph_rect.min.x + from_node.x + 60.0,
                                graph_rect.min.y + from_node.y + 30.0
                            );
                            let to_pos = Pos2::new(
                                graph_rect.min.x + to_node.x,
                                graph_rect.min.y + to_node.y + 30.0
                            );
                            
                            painter.line_segment(
                                [from_pos, to_pos],
                                Stroke::new(2.0, Color32::from_gray(136))
                            );
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
                            "running" => Color32::from_rgb(33, 150, 243),
                            "completed" => Color32::from_rgb(76, 175, 80),
                            "failed" => Color32::from_rgb(244, 67, 54),
                            "cache" => Color32::from_rgb(109, 76, 65),
                            _ => Color32::from_rgb(34, 34, 34),
                        };
                        
                        painter.rect_filled(node_rect, 12.0, node_color);
                        painter.rect_stroke(node_rect, 12.0, Stroke::new(2.0, Color32::from_gray(68)));
                        
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
                            node_clicked = Some(node.id.clone());
                        }
                        
                        if node_response.dragged() {
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
                        
                        if ui.button("Remove Node").clicked() {
                            should_remove_node = true;
                        }
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
        ui.heading("Live Logs");
        
        let state = self.state.lock().unwrap();
        
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for log in &state.live_logs {
                    ui.label(log);
                }
            });
    }
}
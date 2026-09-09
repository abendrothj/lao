use std::collections::HashMap;
use std::fs;

use lao_plugin_api::{PluginInputType, PluginOutputType};

use crate::plugins::*;
use crate::workflow_types::*;

pub fn load_workflow_yaml(path: &str) -> Result<Workflow, String> {
    let yaml_str = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let workflow = parse_workflow_yaml(&yaml_str)?;
    validate_workflow_schema(&workflow)?;
    Ok(workflow)
}

/// Parse workflow YAML text, dispatching on `schema_version` (absent or `1` => the
/// original schema; `2` => `crate::workflow_v2`, normalized into the same `Workflow`
/// shape). Split out from `load_workflow_yaml` so normalization can be unit tested
/// without file I/O.
pub fn parse_workflow_yaml(yaml_str: &str) -> Result<Workflow, String> {
    #[derive(serde::Deserialize)]
    struct SchemaVersionProbe {
        #[serde(default)]
        schema_version: Option<u32>,
    }
    let probe: SchemaVersionProbe = serde_yaml::from_str(yaml_str).map_err(|e| e.to_string())?;
    match probe.schema_version {
        None | Some(1) => serde_yaml::from_str::<Workflow>(yaml_str).map_err(|e| e.to_string()),
        Some(2) => {
            let v2 = serde_yaml::from_str::<crate::workflow_v2::WorkflowV2>(yaml_str)
                .map_err(|e| e.to_string())?;
            crate::workflow_v2::normalize_v2(v2)
        }
        Some(other) => Err(format!(
            "unsupported workflow schema_version {} (supported: 1, 2)",
            other
        )),
    }
}

pub fn validate_workflow_schema(workflow: &Workflow) -> Result<(), String> {
    const UNSUPPORTED_FIELDS: &[&str] = &[
        "on_success",
        "on_failure",
        "input_modality",
        "output_modality",
    ];

    for (idx, step) in workflow.steps.iter().enumerate() {
        let Some(mapping) = step.params.as_mapping() else {
            continue;
        };

        for field in UNSUPPORTED_FIELDS {
            if mapping.contains_key(serde_yaml::Value::String((*field).to_string())) {
                return Err(format!(
                    "Unsupported workflow field '{}' in step {}. This field is not part of the production schema.",
                    field,
                    idx + 1
                ));
            }
        }
    }

    Ok(())
}

pub fn build_dag(steps: &[WorkflowStep]) -> Result<Vec<DagNode>, String> {
    let mut nodes = Vec::new();
    for (index, step) in steps.iter().enumerate() {
        let mut parents = Vec::new();
        if let Some(input_from) = &step.input_from {
            parents.push(input_from.clone());
        }
        if let Some(depends_on) = &step.depends_on {
            parents.extend(depends_on.clone());
        }
        let step_id = format!("step{}", index + 1);
        nodes.push(DagNode {
            id: step_id,
            step: step.clone(),
            parents,
        });
    }
    Ok(nodes)
}

pub fn topo_sort(nodes: &[DagNode]) -> Result<Vec<String>, String> {
    let mut visited = std::collections::HashSet::new();
    let mut visiting = std::collections::HashSet::new();
    let mut order = Vec::new();
    let node_map: HashMap<String, &DagNode> = nodes.iter().map(|n| (n.id.clone(), n)).collect();

    fn visit(
        n: &DagNode,
        map: &HashMap<String, &DagNode>,
        visited: &mut std::collections::HashSet<String>,
        visiting: &mut std::collections::HashSet<String>,
        order: &mut Vec<String>,
    ) -> Result<(), String> {
        if visiting.contains(&n.id) {
            return Err(format!("Circular dependency detected involving {}", n.id));
        }
        if visited.contains(&n.id) {
            return Ok(());
        }
        visiting.insert(n.id.clone());
        for parent_id in &n.parents {
            if let Some(parent) = map.get(parent_id) {
                visit(parent, map, visited, visiting, order)?;
            }
        }
        visiting.remove(&n.id);
        visited.insert(n.id.clone());
        order.push(n.id.clone());
        Ok(())
    }

    for node in nodes {
        if !visited.contains(&node.id) {
            visit(node, &node_map, &mut visited, &mut visiting, &mut order)?;
        }
    }
    Ok(order)
}

pub fn validate_workflow_types(
    dag: &[DagNode],
    plugin_registry: &PluginRegistry,
) -> Vec<(usize, String)> {
    let mut errors = Vec::new();
    for (i, node) in dag.iter().enumerate() {
        let Some(curr_plugin) = plugin_registry.get(&node.step.run) else {
            errors.push((i, format!("Plugin '{}' not found", node.step.run)));
            continue;
        };

        let (curr_in_ty, curr_out_ty) = primary_io_types(curr_plugin);

        for parent_id in &node.parents {
            if let Some(parent_node) = dag.iter().find(|n| &n.id == parent_id) {
                if let Some(parent_plugin) = plugin_registry.get(&parent_node.step.run) {
                    let (_p_in, p_out) = primary_io_types(parent_plugin);
                    if !types_compatible(p_out.clone(), curr_in_ty.clone()) {
                        errors.push((
                            i,
                            format!(
                                "Type mismatch: parent '{}' outputs {:?} but '{}' expects {:?}",
                                parent_node.step.run, p_out, node.step.run, curr_in_ty
                            ),
                        ));
                    }
                }
            }
        }
        let _ = curr_out_ty;
    }
    errors
}

fn primary_io_types(plugin: &PluginInstance) -> (PluginInputType, PluginOutputType) {
    let caps = plugin.get_capabilities();
    if let Some(cap) = caps.first() {
        (cap.input_type.clone(), cap.output_type.clone())
    } else {
        (PluginInputType::Any, PluginOutputType::Any)
    }
}

fn types_compatible(from: PluginOutputType, to: PluginInputType) -> bool {
    use PluginInputType as In;
    use PluginOutputType as Out;
    matches!(
        (from, to),
        (Out::Any, _)
            | (_, In::Any)
            | (Out::Text, In::Text)
            | (Out::Json, In::Json)
            | (Out::Binary, In::Binary)
            | (Out::File, In::File)
            | (Out::Audio, In::Audio)
            | (Out::Image, In::Image)
            | (Out::Video, In::Video)
            | (Out::Audio, In::File)
            | (Out::Image, In::File)
            | (Out::Video, In::File)
            | (Out::File, In::Audio)
            | (Out::File, In::Image)
            | (Out::File, In::Video)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_dag_simple() {
        let steps = vec![WorkflowStep {
            run: "Echo".to_string(),
            params: serde_yaml::from_str("input: 'hello'").unwrap(),
            retries: None,
            retry_delay: None,
            cache_key: None,
            input_from: None,
            depends_on: None,
            condition: None,
            for_each: None,
        }];

        let dag = build_dag(&steps).unwrap();
        assert_eq!(dag.len(), 1);
        assert_eq!(dag[0].id, "step1");
        assert_eq!(dag[0].parents.len(), 0);
    }

    #[test]
    fn test_build_dag_with_dependencies() {
        let steps = vec![
            WorkflowStep {
                run: "Step1".to_string(),
                params: serde_yaml::Value::Null,
                retries: None,
                retry_delay: None,
                cache_key: None,
                input_from: None,
                depends_on: None,
                condition: None,
                for_each: None,
            },
            WorkflowStep {
                run: "Step2".to_string(),
                params: serde_yaml::Value::Null,
                retries: None,
                retry_delay: None,
                cache_key: None,
                input_from: Some("step1".to_string()),
                depends_on: None,
                condition: None,
                for_each: None,
            },
        ];

        let dag = build_dag(&steps).unwrap();
        assert_eq!(dag.len(), 2);
        assert_eq!(dag[1].parents.len(), 1);
        assert_eq!(dag[1].parents[0], "step1");
    }

    #[test]
    fn test_topo_sort_simple() {
        let steps = vec![
            WorkflowStep {
                run: "A".to_string(),
                params: serde_yaml::Value::Null,
                retries: None,
                retry_delay: None,
                cache_key: None,
                input_from: None,
                depends_on: None,
                condition: None,
                for_each: None,
            },
            WorkflowStep {
                run: "B".to_string(),
                params: serde_yaml::Value::Null,
                retries: None,
                retry_delay: None,
                cache_key: None,
                input_from: Some("step1".to_string()),
                depends_on: None,
                condition: None,
                for_each: None,
            },
        ];

        let dag = build_dag(&steps).unwrap();
        let order = topo_sort(&dag).unwrap();
        assert_eq!(order, vec!["step1", "step2"]);
    }

    #[test]
    fn test_topo_sort_circular_dependency() {
        let steps = vec![
            WorkflowStep {
                run: "A".to_string(),
                params: serde_yaml::Value::Null,
                retries: None,
                retry_delay: None,
                cache_key: None,
                input_from: Some("step2".to_string()),
                depends_on: None,
                condition: None,
                for_each: None,
            },
            WorkflowStep {
                run: "B".to_string(),
                params: serde_yaml::Value::Null,
                retries: None,
                retry_delay: None,
                cache_key: None,
                input_from: Some("step1".to_string()),
                depends_on: None,
                condition: None,
                for_each: None,
            },
        ];

        let dag = build_dag(&steps).unwrap();
        let result = topo_sort(&dag);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Circular dependency"));
    }
}

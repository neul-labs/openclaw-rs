//! Workflow engine (m9m pattern).
//!
//! Execute agent logic as a graph of workflow nodes.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Workflow execution errors.
#[derive(Error, Debug)]
pub enum WorkflowError {
    /// Node not found.
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    /// Branch not found.
    #[error("Branch not found: {0}")]
    BranchNotFound(String),

    /// Node execution failed.
    #[error("Node execution failed: {0}")]
    ExecutionFailed(String),

    /// Invalid workflow configuration.
    #[error("Invalid workflow: {0}")]
    InvalidWorkflow(String),

    /// Cycle detected in workflow.
    #[error("Cycle detected at node: {0}")]
    CycleDetected(String),
}

/// Node execution context.
pub struct NodeContext {
    /// Input data.
    pub input: serde_json::Value,
    /// Node configuration.
    pub config: serde_json::Value,
    /// Shared workflow state.
    pub state: HashMap<String, serde_json::Value>,
}

/// Result of node execution.
pub struct NodeOutput {
    /// Output data.
    pub data: serde_json::Value,
    /// Next node ID (explicit routing).
    pub next: Option<String>,
    /// Branch name (conditional routing).
    pub branch: Option<String>,
}

impl NodeOutput {
    /// Create output that continues to next node.
    #[must_use]
    pub fn continue_with(data: serde_json::Value) -> Self {
        Self {
            data,
            next: None,
            branch: None,
        }
    }

    /// Create output that goes to specific node.
    #[must_use]
    pub fn goto(data: serde_json::Value, node_id: impl Into<String>) -> Self {
        Self {
            data,
            next: Some(node_id.into()),
            branch: None,
        }
    }

    /// Create output that takes a branch.
    #[must_use]
    pub fn branch(data: serde_json::Value, branch_name: impl Into<String>) -> Self {
        Self {
            data,
            next: None,
            branch: Some(branch_name.into()),
        }
    }

    /// Create output that ends the workflow.
    #[must_use]
    pub fn end(data: serde_json::Value) -> Self {
        Self {
            data,
            next: Some("__end__".to_string()),
            branch: None,
        }
    }
}

/// Workflow node trait.
#[async_trait]
pub trait WorkflowNode: Send + Sync {
    /// Node identifier.
    fn id(&self) -> &str;

    /// Node type name.
    fn node_type(&self) -> &str;

    /// Execute the node.
    async fn execute(&self, ctx: NodeContext) -> Result<NodeOutput, WorkflowError>;

    /// Input schema (optional).
    fn input_schema(&self) -> Option<&serde_json::Value> {
        None
    }

    /// Output schema (optional).
    fn output_schema(&self) -> Option<&serde_json::Value> {
        None
    }
}

/// Edge connecting two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEdge {
    /// Source node ID.
    pub from: String,
    /// Target node ID.
    pub to: String,
    /// Condition for this edge (branch name).
    pub condition: Option<String>,
}

/// Workflow definition.
pub struct Workflow {
    /// Workflow ID.
    pub id: String,
    /// Workflow name.
    pub name: String,
    /// Nodes in the workflow.
    pub nodes: Vec<Arc<dyn WorkflowNode>>,
    /// Edges connecting nodes.
    pub edges: Vec<WorkflowEdge>,
    /// Starting node ID.
    pub start_node: String,
}

impl Workflow {
    /// Create a new workflow.
    #[must_use]
    pub fn new(id: impl Into<String>, name: impl Into<String>, start_node: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            nodes: Vec::new(),
            edges: Vec::new(),
            start_node: start_node.into(),
        }
    }

    /// Add a node.
    pub fn add_node(&mut self, node: Arc<dyn WorkflowNode>) {
        self.nodes.push(node);
    }

    /// Add an edge.
    pub fn add_edge(&mut self, from: impl Into<String>, to: impl Into<String>) {
        self.edges.push(WorkflowEdge {
            from: from.into(),
            to: to.into(),
            condition: None,
        });
    }

    /// Add a conditional edge.
    pub fn add_conditional_edge(
        &mut self,
        from: impl Into<String>,
        to: impl Into<String>,
        condition: impl Into<String>,
    ) {
        self.edges.push(WorkflowEdge {
            from: from.into(),
            to: to.into(),
            condition: Some(condition.into()),
        });
    }

    /// Find node by ID.
    #[must_use]
    pub fn find_node(&self, id: &str) -> Option<&Arc<dyn WorkflowNode>> {
        self.nodes.iter().find(|n| n.id() == id)
    }

    /// Find outgoing edges from a node.
    #[must_use]
    pub fn outgoing_edges(&self, node_id: &str) -> Vec<&WorkflowEdge> {
        self.edges.iter().filter(|e| e.from == node_id).collect()
    }
}

/// Workflow execution engine.
pub struct WorkflowEngine {
    max_iterations: usize,
}

impl WorkflowEngine {
    /// Create a new workflow engine.
    #[must_use]
    pub fn new() -> Self {
        Self { max_iterations: 1000 }
    }

    /// Set maximum iterations (cycle protection).
    #[must_use]
    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    /// Execute a workflow.
    ///
    /// # Errors
    ///
    /// Returns error if execution fails or cycle detected.
    pub async fn execute(
        &self,
        workflow: &Workflow,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, WorkflowError> {
        let mut current_node_id = workflow.start_node.clone();
        let mut data = input;
        let mut state = HashMap::new();
        let mut iterations = 0;

        loop {
            iterations += 1;
            if iterations > self.max_iterations {
                return Err(WorkflowError::CycleDetected(current_node_id));
            }

            // Find and execute current node
            let node = workflow
                .find_node(&current_node_id)
                .ok_or_else(|| WorkflowError::NodeNotFound(current_node_id.clone()))?;

            let ctx = NodeContext {
                input: data.clone(),
                config: serde_json::Value::Object(serde_json::Map::new()),
                state: state.clone(),
            };

            let output = node.execute(ctx).await?;
            data = output.data;

            // Determine next node
            let next_node = if let Some(explicit_next) = output.next {
                if explicit_next == "__end__" {
                    break;
                }
                Some(explicit_next)
            } else if let Some(branch) = output.branch {
                // Find edge matching the branch
                workflow
                    .outgoing_edges(&current_node_id)
                    .iter()
                    .find(|e| e.condition.as_ref() == Some(&branch))
                    .map(|e| e.to.clone())
            } else {
                // Take first unconditional edge
                workflow
                    .outgoing_edges(&current_node_id)
                    .iter()
                    .find(|e| e.condition.is_none())
                    .map(|e| e.to.clone())
            };

            match next_node {
                Some(next) => current_node_id = next,
                None => break, // No more nodes
            }
        }

        Ok(data)
    }
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple pass-through node for testing.
pub struct PassthroughNode {
    id: String,
}

impl PassthroughNode {
    /// Create a new passthrough node.
    #[must_use]
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }
}

#[async_trait]
impl WorkflowNode for PassthroughNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> &str {
        "passthrough"
    }

    async fn execute(&self, ctx: NodeContext) -> Result<NodeOutput, WorkflowError> {
        Ok(NodeOutput::continue_with(ctx.input))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_workflow() {
        let mut workflow = Workflow::new("test", "Test Workflow", "node1");

        workflow.add_node(Arc::new(PassthroughNode::new("node1")));
        workflow.add_node(Arc::new(PassthroughNode::new("node2")));
        workflow.add_edge("node1", "node2");

        let engine = WorkflowEngine::new();
        let result = engine
            .execute(&workflow, serde_json::json!({"value": 42}))
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap()["value"], 42);
    }

    #[tokio::test]
    async fn test_workflow_node_not_found() {
        let workflow = Workflow::new("test", "Test", "nonexistent");
        let engine = WorkflowEngine::new();

        let result = engine.execute(&workflow, serde_json::json!({})).await;
        assert!(matches!(result, Err(WorkflowError::NodeNotFound(_))));
    }
}

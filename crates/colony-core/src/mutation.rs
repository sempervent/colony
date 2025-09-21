use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use super::{Op, Pipeline};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeneTag {
    pub tags: Vec<String>,     // e.g., ["crc_all", "dual_run", "kalman_inserted"]
    pub generation: u32,
}

impl GeneTag {
    pub fn new() -> Self {
        Self {
            tags: Vec::new(),
            generation: 0,
        }
    }

    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    pub fn increment_generation(&mut self) {
        self.generation += 1;
    }
}

#[derive(Component, Clone, Debug, Serialize, Deserialize)]
pub struct PipelineGenome {
    pub id: String,
    pub gene: GeneTag,
    pub ops: Vec<Op>,
}

impl PipelineGenome {
    pub fn new(id: String, ops: Vec<Op>) -> Self {
        Self {
            id,
            gene: GeneTag::new(),
            ops,
        }
    }

    pub fn from_pipeline(pipeline: &Pipeline, id: String) -> Self {
        Self {
            id,
            gene: GeneTag::new(),
            ops: pipeline.ops.clone(),
        }
    }

    pub fn to_pipeline(&self) -> Pipeline {
        Pipeline {
            ops: self.ops.clone(),
            mutation_tag: Some(self.gene.tags.clone()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Mutation {
    Insert(Op, usize),
    Replace(usize, Op),
    Remove(usize),
    BranchDualRun { adjudicator: Op },
}

impl Mutation {
    pub fn get_tag(&self) -> String {
        match self {
            Mutation::Insert(op, _) => format!("inserted_{:?}", op),
            Mutation::Replace(_, op) => format!("replaced_{:?}", op),
            Mutation::Remove(_) => "removed_op".to_string(),
            Mutation::BranchDualRun { .. } => "dual_run".to_string(),
        }
    }
}

pub fn apply_mutation(genome: &mut PipelineGenome, mutation: Mutation) -> bool {
    match mutation {
        Mutation::Insert(op, index) => {
            if index <= genome.ops.len() {
                genome.ops.insert(index, op);
                genome.gene.add_tag(mutation.get_tag());
                genome.gene.increment_generation();
                return true;
            }
        }
        Mutation::Replace(index, op) => {
            if index < genome.ops.len() {
                genome.ops[index] = op;
                genome.gene.add_tag(mutation.get_tag());
                genome.gene.increment_generation();
                return true;
            }
        }
        Mutation::Remove(index) => {
            if index < genome.ops.len() {
                genome.ops.remove(index);
                genome.gene.add_tag(mutation.get_tag());
                genome.gene.increment_generation();
                return true;
            }
        }
        Mutation::BranchDualRun { adjudicator } => {
            // Create a dual-run branch by duplicating the pipeline and adding adjudicator
            let original_ops = genome.ops.clone();
            genome.ops.extend(original_ops);
            genome.ops.push(adjudicator);
            genome.gene.add_tag(mutation.get_tag());
            genome.gene.increment_generation();
            return true;
        }
    }
    false
}

pub fn parse_mutation_from_effect(
    effect: &super::black_swan::Effect,
    available_ops: &[Op],
) -> Option<Mutation> {
    match effect {
        super::black_swan::Effect::InsertOp { where_, op, .. } => {
            let op_enum = parse_op_from_string(op)?;
            let index = parse_insertion_index(where_, &[])?;
            Some(Mutation::Insert(op_enum, index))
        }
        super::black_swan::Effect::ReplaceOp { from, to, .. } => {
            let from_op = parse_op_from_string(from)?;
            let to_op = parse_op_from_string(to)?;
            // Find the index of the op to replace
            let index = available_ops.iter().position(|op| std::mem::discriminant(op) == std::mem::discriminant(&from_op))?;
            Some(Mutation::Replace(index, to_op))
        }
        super::black_swan::Effect::RemoveOp { op, .. } => {
            let op_enum = parse_op_from_string(op)?;
            let index = available_ops.iter().position(|o| std::mem::discriminant(o) == std::mem::discriminant(&op_enum))?;
            Some(Mutation::Remove(index))
        }
        super::black_swan::Effect::BranchDualRun { adjudicator, .. } => {
            let adjudicator_op = parse_op_from_string(adjudicator)?;
            Some(Mutation::BranchDualRun { adjudicator: adjudicator_op })
        }
        _ => None,
    }
}

fn parse_op_from_string(op_str: &str) -> Option<Op> {
    match op_str {
        "Decode" => Some(Op::Decode),
        "Fft" => Some(Op::Fft),
        "Kalman" => Some(Op::Kalman),
        "Yolo" => Some(Op::Yolo),
        "Crc" => Some(Op::Crc),
        "CanParse" => Some(Op::CanParse),
        "UdpDemux" => Some(Op::UdpDemux),
        "TcpSessionize" => Some(Op::TcpSessionize),
        "ModbusMap" => Some(Op::ModbusMap),
        "HttpParse" => Some(Op::HttpParse),
        "Export" => Some(Op::Export),
        "GpuPreprocess" => Some(Op::GpuPreprocess),
        "GpuExport" => Some(Op::GpuExport),
        "MaintenanceCool" => Some(Op::MaintenanceCool),
        _ => None,
    }
}

fn parse_insertion_index(where_str: &str, ops: &[Op]) -> Option<usize> {
    match where_str {
        "all_outbound" => Some(ops.len()), // Insert at the end
        "start" => Some(0),
        "end" => Some(ops.len()),
        _ => {
            // Try to parse "after:OpName" format
            if let Some(after_op) = where_str.strip_prefix("after:") {
                let after_op_enum = parse_op_from_string(after_op)?;
                let index = ops.iter().position(|op| std::mem::discriminant(op) == std::mem::discriminant(&after_op_enum))?;
                Some(index + 1)
            } else {
                None
            }
        }
    }
}

pub fn mutation_commit_system(
    mut genomes: Query<&mut PipelineGenome>,
    // TODO: Add event reader for mutation requests
) {
    // This system will be called when mutations need to be applied
    // For now, it's a placeholder for the mutation application logic
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gene_tag_operations() {
        let mut gene = GeneTag::new();
        assert_eq!(gene.generation, 0);
        
        gene.add_tag("test_tag".to_string());
        assert!(gene.tags.contains(&"test_tag".to_string()));
        
        gene.increment_generation();
        assert_eq!(gene.generation, 1);
    }

    #[test]
    fn test_pipeline_genome_creation() {
        let ops = vec![Op::Decode, Op::Kalman, Op::Export];
        let genome = PipelineGenome::new("test_pipeline".to_string(), ops.clone());
        
        assert_eq!(genome.id, "test_pipeline");
        assert_eq!(genome.ops, ops);
        assert_eq!(genome.gene.generation, 0);
    }

    #[test]
    fn test_mutation_insert() {
        let mut genome = PipelineGenome::new("test".to_string(), vec![Op::Decode, Op::Export]);
        let mutation = Mutation::Insert(Op::Kalman, 1);
        
        assert!(apply_mutation(&mut genome, mutation));
        assert_eq!(genome.ops, vec![Op::Decode, Op::Kalman, Op::Export]);
        assert_eq!(genome.gene.generation, 1);
        assert!(genome.gene.tags.contains(&"inserted_Kalman".to_string()));
    }

    #[test]
    fn test_mutation_replace() {
        let mut genome = PipelineGenome::new("test".to_string(), vec![Op::Decode, Op::Export]);
        let mutation = Mutation::Replace(0, Op::Kalman);
        
        assert!(apply_mutation(&mut genome, mutation));
        assert_eq!(genome.ops, vec![Op::Kalman, Op::Export]);
        assert_eq!(genome.gene.generation, 1);
    }

    #[test]
    fn test_mutation_remove() {
        let mut genome = PipelineGenome::new("test".to_string(), vec![Op::Decode, Op::Kalman, Op::Export]);
        let mutation = Mutation::Remove(1);
        
        assert!(apply_mutation(&mut genome, mutation));
        assert_eq!(genome.ops, vec![Op::Decode, Op::Export]);
        assert_eq!(genome.gene.generation, 1);
    }

    #[test]
    fn test_mutation_branch_dual_run() {
        let mut genome = PipelineGenome::new("test".to_string(), vec![Op::Decode, Op::Export]);
        let mutation = Mutation::BranchDualRun { adjudicator: Op::Crc };
        
        assert!(apply_mutation(&mut genome, mutation));
        assert_eq!(genome.ops, vec![Op::Decode, Op::Export, Op::Decode, Op::Export, Op::Crc]);
        assert_eq!(genome.gene.generation, 1);
        assert!(genome.gene.tags.contains(&"dual_run".to_string()));
    }

    #[test]
    fn test_op_parsing() {
        assert_eq!(parse_op_from_string("Decode"), Some(Op::Decode));
        assert_eq!(parse_op_from_string("Kalman"), Some(Op::Kalman));
        assert_eq!(parse_op_from_string("InvalidOp"), None);
    }

    #[test]
    fn test_insertion_index_parsing() {
        let ops = vec![Op::Decode, Op::Kalman, Op::Export];
        
        assert_eq!(parse_insertion_index("start", &ops), Some(0));
        assert_eq!(parse_insertion_index("end", &ops), Some(3));
        assert_eq!(parse_insertion_index("all_outbound", &ops), Some(3));
        assert_eq!(parse_insertion_index("after:Kalman", &ops), Some(2));
        assert_eq!(parse_insertion_index("after:InvalidOp", &ops), None);
    }
}

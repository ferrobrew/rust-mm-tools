use std::{
    collections::{HashMap, HashSet},
    ops::Range,
};

use modular_bitfield::prelude::*;

use crate::adf::XvmFormatConstant;

pub struct XvmConstant {
    pub object: XvmObject,
    pub value: u64,
}

impl<T: std::borrow::Borrow<XvmFormatConstant>> From<T> for XvmConstant {
    fn from(value: T) -> Self {
        let value = value.borrow();
        Self {
            object: XvmObject::from(value.flags),
            value: value.value,
        }
    }
}

#[bitfield]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct XvmObject {
    pub size: B8,
    pub capacity: B8,
    pub object_type: XvmObjectType,
    pub reserved: B44,
}

impl<T: std::borrow::Borrow<u64>> From<T> for XvmObject {
    fn from(value: T) -> Self {
        Self::from_bytes(value.borrow().to_le_bytes())
    }
}

#[derive(Specifier, Clone, Copy, Default, Debug, PartialEq)]
#[bits = 4]
pub enum XvmObjectType {
    #[default]
    None = 0,
    Bool = 1,
    Float = 3,
    String = 4,
    List = 5,
    StructClass = 6,
    StructInstance = 7,
    Function = 9,
    CFunction = 10,
    Uninitialized = 14,
}

#[bitfield]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct XvmInstruction {
    pub operation: XvmOperation,
    pub operand: B11,
}

impl<T: std::borrow::Borrow<u16>> From<T> for XvmInstruction {
    fn from(value: T) -> Self {
        Self::from_bytes(value.borrow().to_le_bytes())
    }
}

#[derive(Specifier, Clone, Copy, Default, Debug, PartialEq)]
#[bits = 5]
pub enum XvmOperation {
    #[default]
    Assert = 0,
    BinaryAnd = 1,
    BinaryOr = 2,
    BinaryAddition = 3,
    BinaryDivision = 4,
    BinaryModulo = 5,
    BinaryMultiply = 6,
    BinarySubtract = 7,
    BuildList = 8,
    Call = 9,
    CompareEqual = 10,
    CompareGreaterThanEqual = 11,
    CompareGreaterThan = 12,
    CompareNotEqual = 13,
    Jump = 14,
    JumpIfFalse = 15,
    LoadAttr = 18,
    LoadConst = 19,
    LoadBool = 20,
    LoadGlobal = 21,
    LoadLocal = 22,
    LoadSubscript = 23,
    Pop = 24,
    Print = 25,
    Return = 26,
    StoreAttr = 27,
    StoreLocal = 28,
    StoreSubscript = 29,
    UnaryNot = 30,
    UnaryNegate = 31,
}

impl XvmOperation {
    pub const fn is_control_flow(&self) -> bool {
        matches!(
            self,
            XvmOperation::Jump | XvmOperation::JumpIfFalse | XvmOperation::Return
        )
    }

    pub fn pop_count(&self) -> u16 {
        self.metadata().pop_count() as u16
    }

    pub fn push_count(&self) -> u16 {
        self.metadata().push_count() as u16
    }

    pub fn is_float(&self) -> bool {
        self.metadata().is_float() != 0
    }

    pub const fn metadata(&self) -> XvmOperationMetadata {
        match self {
            XvmOperation::Assert => XvmOperationMetadata::constant(1, 0, 0),
            XvmOperation::BinaryAnd => XvmOperationMetadata::constant(2, 1, 0),
            XvmOperation::BinaryOr => XvmOperationMetadata::constant(2, 1, 0),
            XvmOperation::BinaryAddition => XvmOperationMetadata::constant(2, 1, 1),
            XvmOperation::BinaryDivision => XvmOperationMetadata::constant(2, 1, 1),
            XvmOperation::BinaryModulo => XvmOperationMetadata::constant(2, 1, 1),
            XvmOperation::BinaryMultiply => XvmOperationMetadata::constant(2, 1, 1),
            XvmOperation::BinarySubtract => XvmOperationMetadata::constant(2, 1, 1),
            XvmOperation::BuildList => XvmOperationMetadata::constant(0, 1, 0),
            XvmOperation::Call => XvmOperationMetadata::constant(0, 0, 0), // TODO: Always pushes?
            XvmOperation::CompareEqual => XvmOperationMetadata::constant(2, 1, 0),
            XvmOperation::CompareGreaterThanEqual => XvmOperationMetadata::constant(2, 1, 1),
            XvmOperation::CompareGreaterThan => XvmOperationMetadata::constant(2, 1, 1),
            XvmOperation::CompareNotEqual => XvmOperationMetadata::constant(2, 1, 0),
            XvmOperation::Jump => XvmOperationMetadata::constant(0, 0, 0),
            XvmOperation::JumpIfFalse => XvmOperationMetadata::constant(1, 0, 0),
            XvmOperation::LoadAttr => XvmOperationMetadata::constant(1, 1, 0),
            XvmOperation::LoadConst => XvmOperationMetadata::constant(0, 1, 0),
            XvmOperation::LoadBool => XvmOperationMetadata::constant(0, 1, 0),
            XvmOperation::LoadGlobal => XvmOperationMetadata::constant(0, 1, 0),
            XvmOperation::LoadLocal => XvmOperationMetadata::constant(0, 1, 0),
            XvmOperation::LoadSubscript => XvmOperationMetadata::constant(2, 1, 0),
            XvmOperation::Pop => XvmOperationMetadata::constant(1, 0, 0),
            XvmOperation::Print => XvmOperationMetadata::constant(0, 0, 0),
            XvmOperation::Return => XvmOperationMetadata::constant(0, 0, 0),
            XvmOperation::StoreAttr => XvmOperationMetadata::constant(2, 0, 0),
            XvmOperation::StoreLocal => XvmOperationMetadata::constant(1, 0, 0),
            XvmOperation::StoreSubscript => XvmOperationMetadata::constant(3, 0, 0),
            XvmOperation::UnaryNot => XvmOperationMetadata::constant(1, 1, 0),
            XvmOperation::UnaryNegate => XvmOperationMetadata::constant(1, 1, 0),
        }
    }
}

#[bitfield]
#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct XvmOperationMetadata {
    pub pop_count: B2,
    pub push_count: B2,
    pub reserved: B3,
    pub is_float: B1,
}

impl XvmOperationMetadata {
    pub const fn constant(pop_count: u8, push_count: u8, is_float: u8) -> Self {
        Self::from_bytes([(pop_count & 3) << 0 | (push_count & 3) << 2 | (is_float & 1) << 7])
    }
}

#[derive(Clone, Default, Debug)]
pub struct XvmControlFlowGraph {
    pub count: usize,
    pub ranges: Vec<Range<u16>>,
    pub targets: HashMap<u16, u16>,
    pub predecessors: Vec<Vec<u16>>,
    pub successors: Vec<Vec<u16>>,
    pub post_order: Vec<u16>,
    pub immediate_dominators: Vec<Option<u16>>,
    pub dominance_frontiers: Vec<Vec<u16>>,
}

impl XvmControlFlowGraph {
    pub fn new(operations: &[(XvmOperation, u16)]) -> Self {
        let mut ranges = Self::compute_ranges(operations);
        let mut targets = Self::compute_targets(&ranges);
        let mut successors = Self::compute_successors(operations, &ranges, &targets);
        let mut predecessors = Self::compute_predecessors(&successors);
        let mut post_order = Self::compute_post_order(&successors);
        Self::eliminate_dead_code(
            operations,
            &mut ranges,
            &mut targets,
            &mut successors,
            &mut predecessors,
            &mut post_order,
        );
        let immediate_dominators = Self::compute_immediate_dominators(&predecessors, &post_order);
        let dominance_frontiers =
            Self::compute_dominance_frontiers(&predecessors, &immediate_dominators);
        let count = ranges.len();

        Self {
            count,
            ranges,
            targets,
            predecessors,
            successors,
            post_order,
            immediate_dominators,
            dominance_frontiers,
        }
    }

    fn compute_ranges(operations: &[(XvmOperation, u16)]) -> Vec<Range<u16>> {
        // First pass, create ranges when encountering control flow
        let mut range_start = 0;
        let mut ranges: Vec<Range<u16>> = operations.iter().enumerate().fold(
            Vec::with_capacity(operations.iter().fold(0, |count, (operation, _)| {
                if operation.is_control_flow() {
                    // Returns have no target, and can create no split
                    if matches!(operation, XvmOperation::Return) {
                        return count + 1;
                    }

                    // Jumps have a target, and thus can create a split
                    return count + 2;
                }
                count
            })),
            |mut ranges, (index, (operation, _))| {
                // Split upon encountering control flow
                if operation.is_control_flow() {
                    let block_end = (index + 1) as u16;
                    ranges.push(range_start..block_end);
                    range_start = block_end;
                }
                ranges
            },
        );
        // Second pass, split ranges based on control flow
        let mut current_range = 0;
        while current_range < ranges.len() {
            let range = &ranges[current_range];
            current_range += 1;

            // Non-control flow operations / returns have no target, and do not create a split
            let (operation, operand) = operations[(range.end - 1) as usize];
            if !operation.is_control_flow() || matches!(operation, XvmOperation::Return) {
                continue;
            }
            // Jumps have a target, and thus can create a split
            else if let Some((index, range)) = ranges
                .iter_mut()
                .enumerate()
                .find(|(_, range)| range.start < operand && operand < range.end)
            {
                let block_end = range.end;
                range.end = operand;
                ranges.insert(index + 1, operand..block_end);

                // Ensure we don't process any blocks twice
                if index < current_range {
                    current_range += 1;
                }
            }
        }
        ranges
    }

    fn compute_targets(ranges: &[Range<u16>]) -> HashMap<u16, u16> {
        ranges.iter().enumerate().fold(
            HashMap::with_capacity(ranges.len()),
            |mut targets, (index, block)| {
                targets.insert(block.start, index as u16);
                targets
            },
        )
    }

    fn compute_successors(
        operations: &[(XvmOperation, u16)],
        ranges: &[Range<u16>],
        targets: &HashMap<u16, u16>,
    ) -> Vec<Vec<u16>> {
        // There are at most two successors per range
        ranges.iter().enumerate().fold(
            Vec::with_capacity(ranges.len()),
            |mut successors, (index, range)| {
                let (operation, operand) = operations[(range.end - 1) as usize];
                let mut result = Vec::with_capacity(2);
                // Implicit fall through
                if !matches!(operation, XvmOperation::Return | XvmOperation::Jump) {
                    result.push((index + 1) as u16);
                }
                // Jumps have a target
                if matches!(operation, XvmOperation::Jump | XvmOperation::JumpIfFalse) {
                    result.push(targets[&operand]);
                }
                successors.push(result);
                successors
            },
        )
    }

    fn compute_predecessors(successors: &[Vec<u16>]) -> Vec<Vec<u16>> {
        // First pass, compute predecessor counts
        let mut counts: Vec<u16> = vec![0u16; successors.len()];
        for predecessor in successors {
            for &successor in predecessor {
                counts[successor as usize] += 1;
            }
        }
        // Second pass, fill predecessor arrays
        let mut predecessors: Vec<Vec<u16>> = counts
            .into_iter()
            .map(|count| Vec::with_capacity(count as usize))
            .collect();
        for predecessor in 0..successors.len() {
            for &successor in &successors[predecessor] {
                predecessors[successor as usize].push(predecessor as u16);
            }
        }
        predecessors
    }

    fn eliminate_dead_code(
        operations: &[(XvmOperation, u16)],
        ranges: &mut Vec<Range<u16>>,
        targets: &mut HashMap<u16, u16>,
        successors: &mut Vec<Vec<u16>>,
        predecessors: &mut Vec<Vec<u16>>,
        post_order: &mut Vec<u16>,
    ) {
        // TODO: use sparse arrays or similar?
        fn remove(
            index: usize,
            ranges: &mut Vec<Range<u16>>,
            successors: &mut Vec<Vec<u16>>,
            predecessors: &mut Vec<Vec<u16>>,
        ) {
            // Destroy range, successors, and predecessors
            ranges.remove(index);
            successors.remove(index);
            predecessors.remove(index);
            // Update successor and predecessor hierarchies
            for blocks in successors.iter_mut().chain(predecessors.iter_mut()) {
                for block in blocks.iter_mut() {
                    if *block < index as u16 {
                        continue;
                    }
                    *block -= 1;
                }
            }
        }

        let visited = post_order.iter().cloned().collect::<HashSet<u16>>();
        // First pass, remove ranges containing dead code, skipping entry block
        for index in (1..predecessors.len()).rev() {
            // Only unvisited blocks are dead code
            if visited.contains(&(index as u16)) {
                continue;
            }
            // Remove block from all predecessor lists
            for &successor in &successors[index] {
                let predecessors = &mut predecessors[successor as usize];
                let Some(index) = predecessors.iter().position(|&x| x == index as u16) else {
                    continue;
                };
                predecessors.swap_remove(index);
            }
            remove(index, ranges, successors, predecessors);
        }
        // No ranges removed
        if ranges.len() == targets.len() {
            return;
        }

        // Second pass, coagulate ranges that were split by dead code, skipping entry block
        for index in (1..predecessors.len()).rev() {
            let predecessor = index - 1;
            if predecessors[index].len() != 1 || predecessor != predecessors[index][0] as usize {
                continue;
            }
            if successors[predecessor].len() != 1 || index != successors[predecessor][0] as usize {
                continue;
            }
            let instruction = operations[(ranges[predecessor].end - 1) as usize].0;
            // Only contiguous blocks not separated by control flow can be coagulated
            if ranges[predecessor].end != ranges[index].start || instruction.is_control_flow() {
                continue;
            }
            ranges[predecessor] = ranges[predecessor].start..ranges[index].end;
            successors[predecessor][0] = successors[index][0];
            // Swap block in all predecessor lists
            for &successor in &successors[index] {
                let predecessors = &mut predecessors[successor as usize];
                let Some(index) = predecessors.iter().position(|&x| x == index as u16) else {
                    continue;
                };
                predecessors[index] = predecessor as u16;
            }
            remove(index, ranges, successors, predecessors);
        }

        // Finally, re-compute targets and post order
        *targets = Self::compute_targets(ranges);
        *post_order = Self::compute_post_order(successors);
    }

    fn compute_post_order(successors: &[Vec<u16>]) -> Vec<u16> {
        let mut post_order: Vec<u16> = Vec::with_capacity(successors.len());
        if !successors.is_empty() {
            let mut visited: Vec<bool> = vec![false; successors.len()];
            let mut stack: Vec<u16> = Vec::with_capacity(successors.len());
            // Stack always begins with entry node
            stack.push(0);
            // Walk each chain to its end
            'outer: while let Some(index) = stack.last().cloned() {
                visited[index as usize] = true;
                for &successor in &successors[index as usize] {
                    if !visited[successor as usize] {
                        stack.push(successor);
                        continue 'outer;
                    }
                }
                post_order.push(index);
                stack.pop();
            }
        }
        post_order
    }

    fn compute_immediate_dominators(
        predecessors: &[Vec<u16>],
        post_order: &[u16],
    ) -> Vec<Option<u16>> {
        // Map node index to reverse post order index for efficient intersection
        let mut indices: Vec<u16> = vec![0; predecessors.len()];
        for (index, &node_index) in post_order.iter().rev().enumerate() {
            indices[node_index as usize] = index as u16;
        }
        // Initialize immediate dominators
        let mut idoms: Vec<Option<u16>> = vec![None; predecessors.len()];
        if !idoms.is_empty() {
            // Ensure entry block always dominates itself
            idoms[0] = Some(0);
            // Iteratively find dominators
            let mut changed = true;
            while changed {
                changed = false;
                // Iterate in reverse post order, skipping the entry node
                for &index in post_order.iter().rev().skip(1) {
                    // Filter predecessors to those that have a defined idom
                    let mut processed = predecessors[index as usize]
                        .iter()
                        .filter(|&&index| idoms[index as usize].is_some());
                    // Fetch the next idom
                    let Some(mut next_idom) = processed.next().cloned() else {
                        // It's possible for predecessors to have no idom defined yet
                        continue;
                    };
                    // Intersect current idom with processed
                    for &predecessor in processed {
                        let mut a = next_idom;
                        let mut b = predecessor;
                        while a != b {
                            while indices[a as usize] > indices[b as usize] {
                                a = idoms[a as usize].expect("processed block must have idom");
                            }
                            while indices[b as usize] > indices[a as usize] {
                                b = idoms[b as usize].expect("processed block must have idom");
                            }
                        }
                        next_idom = b;
                    }
                    // Check if the computed idom has changed
                    let prev_idom = &mut idoms[index as usize];
                    if *prev_idom != Some(next_idom) {
                        *prev_idom = Some(next_idom);
                        changed = true;
                    }
                }
            }
        }
        // If any entry is `None` then it's unreachable
        idoms
    }

    fn compute_dominance_frontiers(
        predecessors: &[Vec<u16>],
        idoms: &[Option<u16>],
    ) -> Vec<Vec<u16>> {
        let mut frontiers: Vec<Vec<u16>> = vec![Vec::new(); predecessors.len()];
        for index in 0..predecessors.len() {
            // Skip unreachable blocks
            let Some(idom) = idoms[index] else {
                continue;
            };
            // Only calculate for join points
            if predecessors[index].len() < 2 {
                continue;
            }
            // For each reachable predecessor
            for &predecessor in &predecessors[index] {
                let mut runner = predecessor;
                // Walk runner up the dominator tree until it reaches idom
                loop {
                    // We've reached the end
                    if runner == idom {
                        break;
                    }
                    // Update frontier
                    let next = index as u16;
                    let frontier = &mut frontiers[runner as usize];
                    if !frontier.contains(&next) {
                        frontier.push(next);
                    }
                    // Skip unreachable blocks
                    let Some(next_runner) = idoms[runner as usize] else {
                        break;
                    };
                    // Update runner
                    if runner == next_runner {
                        break;
                    }
                    runner = next_runner;
                }
            }
        }
        frontiers
    }
}

#[cfg(test)]
mod tests {
    use super::{XvmControlFlowGraph, XvmOperation};

    #[test]
    fn test_cfg_dead_code_simple() {
        let operations: &'static [(XvmOperation, u16)] = &[
            (XvmOperation::Jump, 2),   // 0, 0 - jump over dead code
            (XvmOperation::Assert, 0), // 1, ? - dead code
            (XvmOperation::Return, 0), // 2, 2 - exit
        ];
        let cfg = XvmControlFlowGraph::new(&operations);
        assert_eq!(cfg.count, 2);
        assert_eq!(cfg.ranges, &[0..1, 2..3]);
        assert_eq!(cfg.targets, [(0, 0), (2, 1)].into());
        assert_eq!(cfg.predecessors, &[vec![], vec![0]]);
        assert_eq!(cfg.successors, &[vec![1], vec![]]);
        assert_eq!(cfg.post_order, &[1, 0]);
        assert_eq!(cfg.immediate_dominators, &[Some(0), Some(0)]);
        assert_eq!(cfg.dominance_frontiers, &[vec![], vec![]]);
    }

    #[test]
    fn test_cfg_dead_code_advanced() {
        let operations: &'static [(XvmOperation, u16)] = &[
            (XvmOperation::Assert, 0),      // 0, 0 - no op
            (XvmOperation::Jump, 5),        // 1, 0 - jump over dead code
            (XvmOperation::Assert, 0),      // 2, ? - dead code
            (XvmOperation::JumpIfFalse, 1), // 3, ? - dead code
            (XvmOperation::Jump, 2),        // 4, ? - dead code
            (XvmOperation::Return, 0),      // 5, 2 - exit
        ];
        let cfg = XvmControlFlowGraph::new(&operations);
        assert_eq!(cfg.count, 2);
        assert_eq!(cfg.ranges, &[0..2, 5..6]);
        assert_eq!(cfg.targets, [(0, 0), (5, 1)].into());
        assert_eq!(cfg.predecessors, &[vec![], vec![0]]);
        assert_eq!(cfg.successors, &[vec![1], vec![]]);
        assert_eq!(cfg.post_order, &[1, 0]);
        assert_eq!(cfg.immediate_dominators, &[Some(0), Some(0)]);
        assert_eq!(cfg.dominance_frontiers, &[vec![], vec![]]);
    }

    #[test]
    fn test_cfg_while_loop() {
        let operations: &'static [(XvmOperation, u16)] = &[
            (XvmOperation::Assert, 0),      // 0, 0 - loop header
            (XvmOperation::JumpIfFalse, 4), // 1, 1 - loop condition
            (XvmOperation::Assert, 0),      // 2, 2 - loop body
            (XvmOperation::Jump, 1),        // 3, 2 - loop footer
            (XvmOperation::Return, 0),      // 4, 3 - exit
        ];
        let cfg = XvmControlFlowGraph::new(&operations);
        assert_eq!(cfg.count, 4);
        assert_eq!(cfg.ranges, &[0..1, 1..2, 2..4, 4..5]);
        assert_eq!(cfg.targets, [(0, 0), (1, 1), (2, 2), (4, 3)].into());
        assert_eq!(cfg.predecessors, &[vec![], vec![0, 2], vec![1], vec![1]]);
        assert_eq!(cfg.successors, &[vec![1], vec![2, 3], vec![1], vec![]]);
        assert_eq!(cfg.post_order, &[2, 3, 1, 0]);
        assert_eq!(
            cfg.immediate_dominators,
            &[Some(0), Some(0), Some(1), Some(1)]
        );
        // ... loop body's frontier includes itself ... loop confirmed?
        assert_eq!(cfg.dominance_frontiers, &[vec![], vec![1], vec![1], vec![]]);
    }

    #[test]
    fn test_cfg_do_while_loop() {
        let operations: &'static [(XvmOperation, u16)] = &[
            (XvmOperation::Assert, 0),      // 0, 0 - loop header
            (XvmOperation::Assert, 0),      // 1, 1 - loop body
            (XvmOperation::JumpIfFalse, 1), // 2, 1 - loop condition
            (XvmOperation::Jump, 4),        // 3, 2 - loop footer
            (XvmOperation::Return, 0),      // 4, 3 - exit
        ];
        let cfg = XvmControlFlowGraph::new(&operations);
        assert_eq!(cfg.count, 4);
        assert_eq!(cfg.ranges, &[0..1, 1..3, 3..4, 4..5]);
        assert_eq!(cfg.targets, [(0, 0), (1, 1), (3, 2), (4, 3)].into());
        assert_eq!(cfg.predecessors, &[vec![], vec![0, 1], vec![1], vec![2]]);
        assert_eq!(cfg.successors, &[vec![1], vec![2, 1], vec![3], vec![]]);
        assert_eq!(cfg.post_order, &[3, 2, 1, 0]);
        assert_eq!(
            cfg.immediate_dominators,
            &[Some(0), Some(0), Some(1), Some(2)]
        );
        // ... loop body's frontier includes itself ... loop confirmed?
        assert_eq!(cfg.dominance_frontiers, &[vec![], vec![1], vec![], vec![]]);
    }
}

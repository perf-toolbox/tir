use crate::{OpRef, OpRegionIter};

pub fn dfs_walk<F>(op: OpRef, apply: F)
where
    F: Fn(&OpRef) -> (),
{
    let regions = op.borrow().get_regions();
    let mut stack: Vec<(OpRegionIter, OpRef)> = vec![(regions, op)];

    while !stack.is_empty() {
        {
            let cur_op = stack.last_mut().unwrap();

            if let Some(region) = cur_op.0.next() {
                for b in region.iter() {
                    for op in b.iter() {
                        let regions = op.borrow().get_regions();
                        stack.push((regions, op));
                    }
                }
                continue;
            }
        }

        let cur_op = stack.pop().unwrap();
        apply(&cur_op.1);
    }
}

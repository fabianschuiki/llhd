// Copyright (c) 2016 Fabian Schuiki
#include <llhd.h>

// Removes all blocks from a procedure or function that cannot be reached from
// the entry point.
//
// Algorithm:
// 1) Iterate over all basic blocks in the process. For every block, see whether
//    there are no predecessors to that block. I.e. the block is not the entry
//    point of the process and no instructions (br or wait) point to it. If
//    there are predecessors, skip and continue.
// 2) Remove the block from the procedure and make sure all uses it holds of
//    other blocks (e.g. as labels in br or wait instructions) are dropped.
// 3) Iterate over the list of successors the block had and repeat the procedure
//    for each that has no predecessors.

static void
process_block (llhd_value_t BB) {
	if (llhd_block_is_entry(BB) || llhd_block_has_predecessors(BB))
		return;

	// Obtain a list of successors of the block now, since the unlinking further
	// down would make this function return nothing.
	llhd_value_t *successors;
	unsigned num_successors;
	llhd_block_get_successors(BB, &successors, &num_successors);

	// Remove any uses instructions have on other blocks. This will cause
	// successor blocks to lose this block as their predecessor.
	llhd_value_t I;
	for (I = llhd_block_get_first_inst(BB); I; I = llhd_inst_next(I)) {
		llhd_value_unlink_uses(I);
	}

	// Recur on the successor blocks which might have just lost their only
	// predecessor, thus becoming unreachable.
	unsigned i;
	for (i = 0; i < num_successors; ++i) {
		process_block(successors[i]);
	}

	llhd_free(successors);
}

void
llhd_delete_unreachable_blocks (llhd_value_t unit) {

	// For every unreachable block, remove the uses it has on its successor
	// blocks. This makes unreachable blocks and every other block that is the
	// sole decendant of an unrechable block appear as having no predecessors.
	llhd_value_t BB;
	for (BB = llhd_unit_get_first_block(unit); BB; BB = llhd_block_next(BB)) {
		process_block(BB);
	}

	// Remove every block that has no precedessor.
	llhd_value_t BBn;
	for (BB = llhd_unit_get_first_block(unit); BB; BB = BBn) {
		BBn = llhd_block_next(BB);
		if (!llhd_block_is_entry(BB) && !llhd_block_has_predecessors(BB)) {
			llhd_value_unlink(BB);
			llhd_value_free(BB);
		}
	}
}

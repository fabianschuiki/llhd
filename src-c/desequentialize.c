// Copyright (c) 2016 Fabian Schuiki
#include <llhd.h>

// Algorithm:
// 1) Iterate over all basic blocks. For each drive instruction, calculate the
//    conditions along the control path that lead to the instruction, and gather
//    them in an array.
// 2) Sort the array by the driven signal.
// 3) Iterate over the array, merging subsequent entries for the same signal by
//    combining the conditions with an OR and minimizing the resulting
//    expression.
// 4) If the condition evaluates to 1, remove the signal from the array and
//    skip to the next.
// 5) For every drive instruction of the signal, factor out the minimized
//    condition found above. In the combinatorial process, create a new branch
//    based on the resulting condition, insert the instruction tree to calculate
//    the driven value. Add the corresponding inputs and outputs to the process.
//    In the replacement entity, insert the instructions to calculate the
//    minimized storage condition, and add a register instruction based on that
//    storage condition and the driven value established above.
//    Remove the original drive instructions from their parent.
// 6) Instantiate the combinatorial process in the replacement entity.
// 7) Optimize the combinatorial process and the replacement entity.
// 8) Replace all uses of the original process with the replacement entity,
//    which has an identical interface.

void llhd_desequentialize(llhd_value_t proc) {

}

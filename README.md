# Mofang's Garden

Your objective is to clear the board, leaving one Qi node.

Nodes are only free and selectable if they have three or more contiguous free spaces neighboring them. Spaces off the board
count as free.

Elemental nodes match with the element they destroy. They also match with Change to turn into the next node in the cycle.

Heavenly, Earthly, and Human nodes match in a triplet. They also match with Change: Heavenly -> Earthly -> Human -> Heavenly.
In addition, Heavenly matches with Yang to turn into Yang, and similar with Earthly and Yin. Human matches with any element,
becoming that element. the matched element only requires 2 free neighbors.

Yin and Yang match to form 2 Change.

Change matches with itself.

Qi is only free if it has no neighbors. It matches with any elemental node, turning it into Qi, or cancels with itself.

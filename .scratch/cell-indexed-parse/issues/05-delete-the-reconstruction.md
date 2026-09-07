# 05: Delete the reconstruction

**What to build:** Nothing outside the Parser derives an Expression's Cell positions any more, so
the derivation is removed.

The walk that reconstructed each slot's offset by summing token spelling widths goes. Binding an
Expression's operands against current Source reads the Cell-indexed array and the per-root record
instead. The constructor that let a caller assert a consumed width it never measured goes with it.

After this, one Source has one answer for where an Expression ends, and the thing that enumerates an
Expression's Cells is the thing that reads them. The three sites where the reconstruction
contradicted what the Parser measured cannot recur, because there is no reconstruction.

**Blocked by:** 04

**Status:** ready-for-agent

**Tags:** release/v1

- [ ] The layout reconstruction and the source binding that walked it are deleted.
- [ ] Binding an Expression's operands against current Source goes through the array and the
      per-root record.
- [ ] The constructor that accepts a caller-supplied consumed width is deleted.
- [ ] No consumer outside the Parser computes a Cell position from a token's spelling width.
- [ ] Per-root records still carry each root's operands in order for evaluation, and a Span for
      diagnostics.
- [ ] The Tick benchmark's comparison is run, the allocation per Expression is gone, and the
      result is reported whichever way it falls. The spec claims the path gets cheaper; the
      benchmark settles it.

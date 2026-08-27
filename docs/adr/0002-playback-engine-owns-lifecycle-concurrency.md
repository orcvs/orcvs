# Playback Engine owns lifecycle concurrency

The Playback Engine owns its clock task, synchronization, cancellation, stale-clock protection, and shutdown rather than requiring callers to coordinate those implementation details. It presents a cloneable handle with idempotent start and stop operations, synchronously prevents further Ticks before stop returns, and atomically observes lifecycle state while draining ordered diagnostics; this keeps musical-time safety local to the module and makes its interface the shared caller and test surface.

The Source/Playback seam and output adapter seam remain unchanged. Each Playback run has one fixed Tick period and executes its first Tick immediately; output failures and overruns become diagnostics without rolling back Source writes or stopping later Ticks.

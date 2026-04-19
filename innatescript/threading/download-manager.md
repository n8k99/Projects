# G066 — Download Manager

> The Rosetta Stone's first **fan-out worker pool**. One queue, N workers, parallel execution with emergent load balancing. The first project where **the number of concurrent actors is itself a design parameter**, and the first where failure of one job is isolated from the others.

```yaml
id: G066
title: Download Manager
category: threading
requires: [G062-vending-machine, G065-progress-bar]
provides: [shared-queue, worker-pool, pull-scheduling, fan-out-fan-in, failure-isolation, concurrency-limit]
```

## Insight: The Queue Is the Rendezvous

G065 had one producer and one consumer synchronising through a counter. G066 has **one queue and N consumers synchronising through shared state**. The queue is the rendezvous point: producers don't know which worker will run a job, workers don't know about each other, and a single queue-lock mediates every hand-off.

This is the first Rosetta Stone project with a **shared mutable collection** under concurrent access. The queue must be safe against:

- Two workers popping simultaneously (would return the same job to both).
- A worker popping while the producer is pushing (could see a torn state).
- An observer querying the length while either is mutating (could return garbage).

Every language's answer is the same shape: a mutex around the pop/push operations. The critical section is *small* — a single list mutation — which is why the lock doesn't become a throughput bottleneck even with dozens of workers.

Go's channels are the idiomatic answer: a channel IS a synchronised queue, with `<-ch` and `ch<-` compiling to the same primitives the Rust/Python/CL versions build manually. The Go code doesn't "use a mutex" because the mutex is hidden inside the runtime's channel implementation.

## Insight: Pull Scheduling Balances Load for Free

No dispatcher assigns jobs to workers. Workers **pull** jobs from the queue whenever they're ready. A slow worker (working on a big file, or on a slow disk, or on a congested network) pulls less often; a fast worker pulls more often. The queue drains at the combined rate of all workers.

This is profound. **Load balancing is emergent, not designed.** The system doesn't need to predict which job will be slow, or which worker is heavily loaded. The pull model makes the right decision automatically: idle workers take work; busy workers don't. A bad guess about job size at enqueue time doesn't penalise the overall throughput — a fast worker will blow through many small jobs while a slow worker grinds on its one big one, and both finish around the same time.

Push scheduling (dispatcher-assigns-to-specific-worker) is worse in almost every case. It requires the dispatcher to guess worker load, guess job cost, and correct its guesses as reality diverges from its model. Pull scheduling punts the whole problem to the workers themselves and wins.

Pull is how the noosphere's agent-assignment works. The agent-dispatch queue holds pending choreographies; when an agent finishes a task and becomes idle, it requests the next one. There is no dispatcher making worker-specific routing decisions. G066 is that pattern at minimal scale.

## Insight: Worker Count as a Resource Limit

More workers means more parallelism means more throughput — up to a point. Past that point, more workers means more context-switching, more memory pressure, more open connections, and the system slows down. The optimal worker count for a given workload is a production tuning question. G066 presents it as a constructor parameter: `DownloadManager::new(4)`, `DownloadManager(worker_count=4)`.

This is the first Rosetta Stone project with a **concurrency limit as an explicit resource cap**. Setting it to 1 is legal (degenerate case — serial processing). Setting it to 1000 is legal but almost certainly wrong. Production systems expose this as configuration; the right value depends on the workload and the machine.

Concurrency limits are everywhere in the noosphere: the agent-dispatch pool size, the max-concurrent-IPC-clients the Rust daemon handles, the number of simultaneous vault-document watchers, the parallel-tool-call cap when an LLM wants to spawn many agents. G066 introduces the parameter.

## Insight: Fan-Out, Fan-In — the Map-Reduce Shape

The queue *fans out* work across N workers; the results collection *fans in* the outcomes back into one list. This is the classic **map-reduce** shape, and G066 is its minimal instance in the milestone.

- Map: each worker transforms `Download → DownloadResult` independently.
- Reduce: the aggregator combines all results into one collection (here, a simple list; in map-reduce proper, it could be a sum, a count, a merge-sorted output, etc.).

Every large-scale batch processing pipeline — Hadoop, Spark, Beam, the noosphere's future document-reindex job — has this exact shape. The number of workers tunes parallelism; the reduce step determines what "the answer" is.

## Insight: Failure Isolation Is the Whole Point of Per-Job Transactions

If one download fails, the other workers must keep going, and the other jobs must still run. A worker that crashes on a failed job leaves all remaining jobs orphaned. A manager that aborts on the first failure loses all the successful work.

G066 treats each job as a **transaction**. The worker wraps the simulate call in a try/catch (Python, CL), a `Result` match (Rust, Lean), or error-value propagation (Go); either way, the error is captured **as the outcome of that job** and the worker moves on to the next one.

This is the first Rosetta Stone project where **partial failure is a first-class outcome, not an abort condition**. Every previous project (including G065) treated a failure as a fatal state for the entire operation. G066 promotes failure to a per-job outcome, co-equal with success and cancellation.

Production systems live on this distinction. A batch job with 10,000 tasks, one of which fails, is a report-and-continue case, not a crash-the-pipeline case. The result list is the audit trail: "10,000 attempted, 9,997 succeeded, 3 failed with these specific errors." G066 is that pattern at minimum scale.

## Insight: Cancellation Now Has Scope

G065 had one worker and one cancel flag. G066 has N workers — and **cancellation applies to all of them at once**. A single `cancel()` call from any thread causes every running worker to check its flag on the next iteration and start producing `Cancelled` outcomes (or exit, for jobs not yet claimed).

The scope of cancellation is **the whole manager**. Individual jobs are not separately cancellable — you either cancel the batch or you let it finish. A finer-grained API (cancel-job-by-id) would be possible but invites complexity: the job might already have started, might have partially completed, might be in flight over the network. G066 makes the coarse choice and notes that finer control is a separate design problem.

Go's `context.Context` is the cleanest encoding: the context carries the cancellation signal through every function call in the pool, and `<-ctx.Done()` is the checkpoint. Rust tokio's `CancellationToken` and Python's `threading.Event` play the same role. The pattern is universal; the ergonomics differ.

## Choreographic Case: Parallel Vault Reindex

```innate
(@reindex-vault){
  @notes <- @vault/all-notes
  @mgr <- @download/manager{worker_count: 8}
  @for note in @notes {
    @mgr/enqueue{job: {url: "file://" + note.path, expected_bytes: note.size}}
  }

  @results <- @mgr/run-with{simulate: @note/reparse-and-upsert}

  @report <- @results/split-by-status
  @print "reindex: ${@report.completed} ok, ${@report.failed} failed"

  @for r in @report.failed {
    @log{level: "warn", url: @r.url, error: @r.error}    ;; continue despite failures
  }
}
```

The choreography is natural because the domain maps cleanly: enqueue every note as a job, a pool of 8 workers reparses them in parallel, failures are logged but don't abort the run, the summary is the manager's result list.

## Structures

```innate
(defstruct download
  url             : String
  expected-bytes  : Int)

(defstruct download-result
  url    : String
  status : "completed" | "cancelled" | "failed"
  bytes  : Int?
  error  : String?)

(defstruct manager
  worker-count : Int
  queue        : [Download]     ;; protected
  results      : [DownloadResult]
  cancelled    : Bool)
```

## Resolver Natives

```innate
@download/manager{worker_count}                   -> Manager
@manager/enqueue{manager, job}                    -> Unit
@manager/cancel{manager}                          -> Unit
@manager/run-with{manager, simulate}              -> [DownloadResult]   ;; blocks
@manager/queued-count{manager}                    -> Int
@manager/result-count{manager}                    -> Int
```

## Demo

```innate
(@demo){
  @mgr <- @download/manager{worker_count: 4}
  @for i in 0..12 {
    @mgr/enqueue{job: {url: "https://example.org/file-${i}.bin",
                       expected_bytes: 100_000 * (i + 1)}}
  }
  @results <- @mgr/run-with{simulate: @fake/download}    ;; ~0.17s across 4 workers
  @completed <- @results.filter(.status == "completed").length     ;; -> 11
  @failed    <- @results.filter(.status == "failed").length         ;; -> 1
}
```

## Where

The job queue MUST be protected by a mutex, a channel, or an equivalent synchronisation primitive — a naked list under concurrent pop/push is a data race. Worker count MUST be >= 1 at construction — zero workers is a silent deadlock. Each job MUST be wrapped in per-job error handling — a worker that lets an exception propagate out of its loop kills itself and leaves remaining jobs orphaned; the worker MUST convert failures into outcomes and continue. Cancellation MUST apply to the whole pool uniformly — there is no per-job cancel, only "cancel the batch." The results collection MUST be in completion order, not enqueue order — workers run in parallel and the enqueue order has no operational meaning once the queue has more than one worker. The simulate function MUST be pure with respect to the manager — it MUST NOT touch the queue, the results, or the cancel flag; all manager state is the worker's to manage.

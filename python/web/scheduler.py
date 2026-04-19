"""Scheduled Auto Login and Action — cron-like task scheduler.

Tasks are data: (name, schedule, action, credential_ref). The scheduler
owns tasks and run history; `tick(now)` fires due tasks through a
pluggable dispatch function. Same abstract-transport pattern as G072 —
tests use fakes, production plugs in real handlers.

Catch-up policy: far-ahead tick fires each due task ONCE (cron-style
skip-missed), not once per missed boundary. Avoids thundering herds
after downtime.
"""

from __future__ import annotations
from dataclasses import dataclass, field
from enum import Enum
from typing import Callable


DAY_MS = 86_400_000


class ScheduleKind(Enum):
    EVERY_MS = "every_ms"
    AT_MS = "at_ms"
    DAILY = "daily"


@dataclass(frozen=True)
class Schedule:
    kind: ScheduleKind
    interval_ms: int = 0
    at_ms: int = 0
    hour: int = 0
    minute: int = 0

    @classmethod
    def every_ms(cls, ms: int) -> "Schedule":
        return cls(kind=ScheduleKind.EVERY_MS, interval_ms=ms)

    @classmethod
    def at_ms(cls, t: int) -> "Schedule":
        return cls(kind=ScheduleKind.AT_MS, at_ms=t)

    @classmethod
    def daily(cls, hour: int, minute: int) -> "Schedule":
        return cls(kind=ScheduleKind.DAILY, hour=hour, minute=minute)


@dataclass
class Task:
    id: int
    name: str
    schedule: Schedule
    action: str
    credential_ref: str | None
    next_fire_ms: int
    enabled: bool = True


class RunStatus(Enum):
    SUCCEEDED = "succeeded"
    FAILED = "failed"


@dataclass(frozen=True)
class Run:
    task_id: int
    started_ms: int
    finished_ms: int
    status: RunStatus
    message: str


@dataclass
class DispatchContext:
    task: Task
    now_ms: int


def _next_daily(base_ms: int, hour: int, minute: int) -> int:
    day = base_ms // DAY_MS
    day_start = day * DAY_MS
    target_in_day = hour * 3_600_000 + minute * 60_000
    candidate = day_start + target_in_day
    if candidate > base_ms:
        return candidate
    return candidate + DAY_MS


def _compute_initial_fire(sched: Schedule, base_ms: int) -> int:
    if sched.kind is ScheduleKind.EVERY_MS:
        return base_ms + sched.interval_ms
    if sched.kind is ScheduleKind.AT_MS:
        return sched.at_ms
    return _next_daily(base_ms, sched.hour, sched.minute)


def _schedule_next(sched: Schedule, prev_fire_ms: int, now_ms: int) -> tuple[int, bool]:
    if sched.kind is ScheduleKind.EVERY_MS:
        nxt = prev_fire_ms + sched.interval_ms
        while nxt <= now_ms:
            nxt += sched.interval_ms
        return nxt, False
    if sched.kind is ScheduleKind.AT_MS:
        return 2**63 - 1, True  # far future, disabled
    return _next_daily(now_ms, sched.hour, sched.minute), False


class Scheduler:
    def __init__(self) -> None:
        self._tasks: list[Task] = []
        self._history: list[Run] = []
        self._next_id = 1

    def add_task(self, name: str, schedule: Schedule, action: str,
                 credential_ref: str | None = None,
                 first_fire_base_ms: int = 0) -> int:
        tid = self._next_id
        self._next_id += 1
        self._tasks.append(Task(
            id=tid, name=name, schedule=schedule, action=action,
            credential_ref=credential_ref,
            next_fire_ms=_compute_initial_fire(schedule, first_fire_base_ms),
        ))
        return tid

    def remove_task(self, tid: int) -> bool:
        for i, t in enumerate(self._tasks):
            if t.id == tid:
                del self._tasks[i]
                return True
        return False

    def set_enabled(self, tid: int, enabled: bool) -> bool:
        for t in self._tasks:
            if t.id == tid:
                t.enabled = enabled
                return True
        return False

    def tasks(self) -> list[Task]:
        return list(self._tasks)

    def get_task(self, tid: int) -> Task | None:
        return next((t for t in self._tasks if t.id == tid), None)

    def history(self) -> list[Run]:
        return list(self._history)

    def history_for(self, task_id: int) -> list[Run]:
        return [r for r in self._history if r.task_id == task_id]

    def next_fires(self) -> dict[int, int]:
        return {t.id: t.next_fire_ms for t in self._tasks}

    def tick(self, now_ms: int,
             dispatch: Callable[[DispatchContext], tuple[bool, str]]) -> list[Run]:
        """Fire every task whose next_fire_ms <= now_ms.
        dispatch returns (succeeded, message)."""
        due = [t for t in self._tasks if t.enabled and t.next_fire_ms <= now_ms]
        new_runs: list[Run] = []
        for task in due:
            ctx = DispatchContext(task=task, now_ms=now_ms)
            succeeded, msg = dispatch(ctx)
            run = Run(
                task_id=task.id, started_ms=now_ms, finished_ms=now_ms,
                status=RunStatus.SUCCEEDED if succeeded else RunStatus.FAILED,
                message=msg,
            )
            self._history.append(run)
            new_runs.append(run)
            next_fire, disable = _schedule_next(task.schedule, task.next_fire_ms, now_ms)
            task.next_fire_ms = next_fire
            if disable:
                task.enabled = False
        return new_runs


if __name__ == "__main__":
    s = Scheduler()
    s.add_task("hourly backup",
               Schedule.every_ms(60_000),
               "backup-vault",
               credential_ref="backup-host",
               first_fire_base_ms=0)
    s.add_task("morning report",
               Schedule.daily(hour=9, minute=30),
               "send-report",
               credential_ref="email",
               first_fire_base_ms=0)
    s.add_task("one-off migration",
               Schedule.at_ms(150_000),
               "run-migration",
               credential_ref=None,
               first_fire_base_ms=0)

    def dispatch(ctx: DispatchContext) -> tuple[bool, str]:
        return (True, f"{ctx.task.name} executed (cred={ctx.task.credential_ref})")

    for now in [30_000, 60_000, 120_000, 150_000, 180_000, 9 * 3_600_000 + 30 * 60_000]:
        runs = s.tick(now, dispatch)
        if runs:
            print(f"t={now/1000:>8.1f}s: {len(runs)} run(s)")
            for r in runs:
                print(f"    - {r.message} [{r.status.value}]")

    print()
    print(f"total runs in history: {len(s.history())}")
    migration = s.get_task(3)
    print(f"migration task enabled after firing: {migration.enabled} (expected False)")

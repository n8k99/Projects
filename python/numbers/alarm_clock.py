"""Alarm Clock — schedule and trigger time-based alerts."""

from datetime import datetime


class AlarmClock:
    """Manage a collection of time-based alarms.

    Each alarm has an hour, minute, second, and optional label.
    Calling check() compares pending alarms against the current system
    time and returns any that have triggered.
    """

    def __init__(self):
        self._alarms = []

    def set_alarm(self, hour: int, minute: int, second: int = 0,
                  label: str = "") -> dict:
        """Schedule an alarm for the given time today.

        Returns the alarm dict that was added.
        """
        if not (0 <= hour <= 23 and 0 <= minute <= 59 and 0 <= second <= 59):
            raise ValueError(f"invalid time: {hour:02d}:{minute:02d}:{second:02d}")
        alarm = {"hour": hour, "minute": minute, "second": second,
                 "label": label, "triggered": False}
        self._alarms.append(alarm)
        return alarm

    def check(self) -> list:
        """Check all pending alarms against the current system time.

        Returns a list of alarms that have just triggered (newly fired
        this call). Triggered alarms are marked so they won't fire again.
        """
        now = datetime.now()
        triggered = []
        for alarm in self._alarms:
            if alarm["triggered"]:
                continue
            alarm_seconds = alarm["hour"] * 3600 + alarm["minute"] * 60 + alarm["second"]
            now_seconds = now.hour * 3600 + now.minute * 60 + now.second
            if now_seconds >= alarm_seconds:
                alarm["triggered"] = True
                triggered.append(alarm)
        return triggered

    def pending(self) -> list:
        """Return all alarms that have not yet triggered."""
        return [a for a in self._alarms if not a["triggered"]]

    def cancel(self, label: str) -> bool:
        """Cancel the first pending alarm with the given label.

        Returns True if an alarm was cancelled, False otherwise.
        """
        for i, alarm in enumerate(self._alarms):
            if alarm["label"] == label and not alarm["triggered"]:
                self._alarms.pop(i)
                return True
        return False


def _format_time(h, m, s):
    return f"{h:02d}:{m:02d}:{s:02d}"


if __name__ == "__main__":
    clock = AlarmClock()

    # Set an alarm in the past — should trigger immediately on check.
    clock.set_alarm(0, 0, 0, label="midnight")
    # Set an alarm for the end of the day — should stay pending.
    clock.set_alarm(23, 59, 59, label="end-of-day")
    # Set one to cancel.
    clock.set_alarm(12, 0, 0, label="noon")

    print("Pending alarms:")
    for a in clock.pending():
        print(f"  {_format_time(a['hour'], a['minute'], a['second'])} [{a['label']}]")

    print(f"\nCancelling 'noon': {clock.cancel('noon')}")
    print(f"Cancelling 'nonexistent': {clock.cancel('nonexistent')}")

    print(f"\nPending after cancel: {len(clock.pending())}")

    triggered = clock.check()
    print(f"\nTriggered alarms: {len(triggered)}")
    for a in triggered:
        print(f"  {_format_time(a['hour'], a['minute'], a['second'])} [{a['label']}]")

    print(f"Still pending: {len(clock.pending())}")

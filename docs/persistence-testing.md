# Persistence acceptance test

The automated `bot-database` test uses disposable on-disk databases. It closes
and reopens the database, verifies persisted records and reminder delivery/retry
state, and opens a separate SQLite snapshot to verify its data and integrity.
It does not exercise the deployed Docker volume, Discord delivery, or the
operator's actual backup procedure.

Run it locally on Windows:

```powershell
./scripts/dev.ps1 --% test --locked -p bot-database disk_persistence_and_snapshot_restore
```

## Live restart test

Use a private test channel and your existing deployment. Ensure Utility and
Music are enabled there. Leave Utility enabled throughout reminder tests.

1. Create a reminder with `/remind`, using duration `5m` and a recognizable
   message such as `persistence acceptance`. Note its ID from `/reminders`.
2. Bookmark a test message through **Apps → Bookmark Message**, and add a note
   with `/bookmark note`. Verify it using `/bookmark list`.
3. Queue two supported tracks and use `/playlist save persistence-test`.
   Record the playlist's track order. Saved playlists persist; the active queue
   and voice connection are runtime state and are not expected to resume.
4. Record `/module list`, `/prefix show`, and `/permissions list`. If you need
   to test a new setting, change only an optional module or prefix in the test
   server and record its old value. For moderation history, record an existing
   `/case` number or issue a test warning to a consenting test account.
5. On the deployment host, from the Compose directory, restart only the bot:

   ```sh
   docker compose restart bot
   docker compose ps bot
   docker compose logs --since 2m bot
   ```

6. Wait for `Bot ready`. Confirm the same reminder ID/message remains in
   `/reminders` before its due time, the bookmark/note remains, and the recorded
   settings, rules, and moderation case are unchanged. Use `/playlist list`
   and `/playlist play persistence-test` to verify the saved tracks and order.
7. Confirm the reminder arrives in its original channel and disappears from
   `/reminders` after successful delivery. Allow a scheduler tick (15 seconds)
   after the due time, plus Discord/network latency.

## Reminder due while offline

Create another reminder due in two minutes. Stop only the bot with
`docker compose stop bot`, wait until the reminder is overdue, then run
`docker compose start bot`. After readiness, it should deliver on a scheduler
tick. Verify it disappears from `/reminders` after delivery. The delivery
contract is at least once: a crash between sending and recording success may
produce a duplicate. An ordinary clean restart should not repeat a reminder
already recorded as delivered.

If using a recurring reminder, confirm it retains its ID and interval after
restart and schedules its next occurrence after delivery. Cancel the test
reminder afterward with `/reminder cancel`.

Restore any test settings and delete the test playlist/bookmark afterward.
Keep a brief record of the deployed revision, results, and any failures.
Never remove the data volume during these tests.

## Backup recovery is a separate check

Follow the README's online backup procedure, then restore that backup into a
disposable database or volume and validate its integrity, migrations, and
expected records. Do not overwrite the live database. The local snapshot test
does not certify a backup produced on the deployment host.

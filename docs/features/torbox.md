# TorBox

The TorBox module generates private, temporary links for safe files in cached magnets, or creates an HLS stream for an eligible cached video. Enable it with `/module enable torbox`; it is disabled by default and intentionally slash-only because magnet inputs can be sensitive.

## Account and access setup

Set the operator's `TORBOX_API_KEY` in the private environment/Compose `.env`. The bot shares that account and its rate limits among authorized server users, so restrict the module to trusted roles with Core's `/permissions` before enabling it. The module needs outbound HTTPS and no extra executable or container. There are no TorBox-command TOML settings. Music's `TORBOX_ENABLED` controls the separate music provider, not this module.

TorBox features depend on your subscription. The stream-creation API is documented by TorBox as Pro-only; a lower subscription may support cached downloads while rejecting streams. The bot handles that rejection without exposing the API key. See the [configuration reference](../reference/configuration.md).

## Commands and behavior

```text
/torbox download magnet:<your-authorized-cached-magnet>
/torbox stream magnet:<your-authorized-cached-video-magnet>
```

Use your own authorized magnet in the private slash field. An optional one-based `file_number` bypasses the picker. With multiple eligible files, choose from up to 250 entries, 25 per page. Only the requester can use the picker and it expires after five minutes. Potentially infected files are excluded; ZIP download is offered only when every file is reported safe.

For ZIPs, choose **Prepare ZIP download**, then **Open ZIP download** after the response updates. Discord cannot automatically open a browser download. File links expire in approximately three hours, so start promptly and regenerate an expired link. `/torbox stream` returns a temporary HLS link for a compatible player; it does not open the signed-in TorBox dashboard player. See the [command reference](../reference/commands.md).

## Limits and common problems

Uncached magnets are rejected without starting a download. A service cache miss is not a reason to repeatedly retry a request or bypass cached-only behavior. Unknown files, unavailable safe metadata, account restrictions and API throttling can prevent a link. Private results still contain shareable temporary access: do not forward them to untrusted recipients or post them in logs.

If download works but stream fails, check the subscription and whether the chosen file is an eligible video. Expired pickers need a new command; expired links need regeneration. The command has a ten-second per-user cooldown. Live Pro streaming and the new interactive rendering still require complete Discord acceptance; the implemented API path is not a claim that every account supports it.

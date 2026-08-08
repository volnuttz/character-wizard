# Portable character sharing

Character share codes are a compact, copy-and-paste representation of the
canonical JSON record. JSON remains the source of truth; importing a code writes
a normal, validated JSON file.

## Commands

Export accepts either a character name from the default collection or an
explicit JSON path:

```console
character-wizard export legolas
character-wizard export ./saves/legolas.json
character-wizard export legolas --directory ./party
```

Export writes only the share code to standard output, so it can be copied or
piped directly:

```console
code=$(character-wizard export legolas)
character-wizard import "$code"
```

Import defaults to `./<character-name>.json` in the current directory. Select a
different collection or exact destination when needed:

```console
character-wizard import <code> --directory ./party
character-wizard import <code> --output ./saves/legolas.json
```

Import refuses to replace an existing destination. Pass `--force` only when an
overwrite is intentional.

Characters tied to a campaign data pack require the exact pack when exporting
or importing:

```console
character-wizard export moon-warden --data ./my-campaign
character-wizard import <code> --data ./my-campaign
```

## Version 1 format

A version-1 code has this shape:

```text
cw1:<unpadded-base64url-encoded-compact-character-json>
```

The prefix versions the envelope independently from character JSON. Future
incompatible envelopes must use a new prefix; unknown versions are rejected.
Version 1 deliberately avoids compression and external dependencies.

Inputs are limited to a 256 KiB encoded payload and 192 KiB after decoding. The
decoder accepts only the unpadded URL-safe Base64 alphabet, requires UTF-8 JSON,
and applies the complete canonical character and exact data-pack validation
before writing anything.

## Security and privacy

Treat every received code as untrusted input. A share code is encoding, not
encryption, authentication, or a digital signature. Anyone holding it can read
and modify its character data, including backstory or other personal text.
Inspect imported characters before relying on them, and share codes only through
channels appropriate for their contents.

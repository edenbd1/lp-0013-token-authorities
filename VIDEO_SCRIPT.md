# LP-0013 — Video Script (~4 min)

- Duration: **~4 minutes**
- Language: English
- `ACTION` = what you do on screen
- `SAY` = what you say out loud

---

## Pre-flight (~2 min prep)

### Open 2 browser tabs

**TAB A (repo):**
https://github.com/edenbd1/lp-0013-token-authorities

**TAB B (crates.io):**
https://crates.io/crates/lez-approval

### Terminal

```bash
cd /path/to/logos-execution-zone   # the LEZ fork
clear
```

Enlarge font. Start screen recording (QuickTime or OBS).

---

## SCENE 1 — Intro (0:00 – 0:30)

**ACTION:** Terminal visible, empty

**SAY:**

> "Hi, I'm Eden. This is my submission for Logos Lambda Prize LP-0013 — Token Program Improvements: Authorities."

> "It adds a rotatable mint authority model to the LEZ token program. You can set a mint authority at token creation, mint tokens gated by that authority, and permanently revoke it to fix the supply. The approval library is published on crates.io as a reusable primitive."

---

## SCENE 2 — Repo tour (0:30 – 1:30)

**ACTION:** Switch to TAB A (GitHub repo). Scroll through README.

**SAY:**

> "Here's the repo. Four crates: lez-approval is the RFP-001 agnostic library — 13 unit tests, zero dependencies beyond borsh and serde. Token-authority-core defines the instruction types and the new FungibleWithAuthority variant. Token-authority-program has the handlers — 50 unit tests. And the SDK re-exports everything."

**ACTION:** Click into `crates/lez-approval/src/lib.rs`

**SAY:**

> "The authority is a simple wrapper around an optional 32-byte account ID. Gate checks if the signer matches. Rotate atomically replaces the admin. Revoke sets it to None permanently — terminal, can't be reversed. This is provably enforced by the RISC0 guest."

**ACTION:** Click into `examples/` folder

**SAY:**

> "Three examples: fixed supply — mint then revoke. Variable supply — mint as needed. And config PDA gating — this one demonstrates RFP-001 section 4, showing the library can gate any program's privileged instructions, not just token minting."

**ACTION:** Switch to TAB B (crates.io)

**SAY:**

> "The approval library is published on crates.io. Add lez-approval equals zero point one to your Cargo.toml and you get the full authority primitive without forking anything."

---

## SCENE 3 — Live demo (1:30 – 3:00)

**ACTION:** Switch to terminal. Type slowly:

```bash
bash /path/to/lp-0013-token-authorities/scripts/demo.sh
```

**SAY** (when banner appears):

> "RISC0 dev mode equals zero — real cryptographic proofs, no shortcuts."

**ACTION:** Wait for step 1 to complete

**SAY:**

> "Step one creates a token called DEMO with initial supply 1000. The authority is set to the definition account owner. You can see supply equals 1000, nonce equals 1 — confirmed on the sequencer via JSON-RPC."

**ACTION:** Wait for step 2

**SAY:**

> "Step two mints 500 tokens. Supply is now 1500, nonce 2. The mint was gated by the authority — the guest verified that the definition account is authorized AND that its account ID matches the stored authority."

**ACTION:** Wait for step 3

**SAY:**

> "Step three revokes the authority permanently. Nonce increments to 3. The authority is now None — a terminal state that cannot be reversed."

**ACTION:** Wait for step 4

**SAY:**

> "Step four tries to mint after revocation. The wallet sends the transaction, but the sequencer rejects it. You can see in the logs: Guest panicked — Renounced, authority has been permanently revoked. The supply stays at 1500."

---

## SCENE 4 — Architecture + closing (3:00 – 3:45)

**ACTION:** Switch to TAB A, open `docs/architecture.md`

**SAY:**

> "Quick architecture recap. The authority is verified cryptographically at the account level — the signer is a pre-state account with is-authorized checked by the NSSA runtime. It's never passed as spoofable instruction data. Rotation and revocation are atomic single-field writes — no intermediate state possible."

> "The new FungibleWithAuthority variant is a separate enum variant from Fungible, so existing token definitions are completely unaffected. Regular Mint is explicitly rejected on authority tokens — you must use MintWithAuthority."

**SAY:**

> "72 tests total, CI green, two IDLs — hand-authored and SPEL-generated, CU measurements from the risc0 executor, and the library on crates.io. Thanks for reviewing."

---

## End

Stop recording. Export.

<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  type Profile = string;

  let profiles = $state<Profile[]>([]);
  let loading = $state(true);
  let error = $state("");
  let busy = $state("");
  let renameDraft = $state<Record<string, string>>({});
  let newName = $state("");

  async function refresh() {
    loading = true;
    error = "";
    try {
      profiles = await invoke<string[]>("list_profiles");
      const drafts: Record<string, string> = {};
      for (const p of profiles) drafts[p] = p;
      renameDraft = drafts;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function open(name: string) {
    busy = `open:${name}`;
    error = "";
    try {
      await invoke("open_profile", { name });
    } catch (e) {
      error = String(e);
    } finally {
      busy = "";
    }
  }

  async function rename(from: string) {
    const to = (renameDraft[from] ?? "").trim();
    if (!to || to === from) return;
    busy = `rename:${from}`;
    error = "";
    try {
      await invoke("rename_profile", { from, to });
      await refresh();
    } catch (e) {
      error = String(e);
    } finally {
      busy = "";
    }
  }

  async function remove(name: string) {
    if (!confirm(`Delete account "${name}"?\nSession data will be removed.`)) return;
    busy = `del:${name}`;
    error = "";
    try {
      await invoke("delete_profile", { name });
      await refresh();
    } catch (e) {
      error = String(e);
    } finally {
      busy = "";
    }
  }

  async function addNamed() {
    const name = newName.trim();
    if (!name) return;
    busy = "add";
    error = "";
    try {
      await invoke("create_profile", { name });
      newName = "";
      await refresh();
      await invoke("open_profile", { name });
    } catch (e) {
      error = String(e);
    } finally {
      busy = "";
    }
  }

  async function addNext() {
    busy = "add";
    error = "";
    try {
      const name = await invoke<string>("create_next_profile");
      await refresh();
      await invoke("open_profile", { name });
    } catch (e) {
      error = String(e);
    } finally {
      busy = "";
    }
  }

  onMount(refresh);
</script>

<main>
  <header>
    <h1>Accounts</h1>
    <p class="sub">Rename, delete, or open WhatsApp profiles. Each has its own session.</p>
  </header>

  {#if error}
    <div class="err" role="alert">{error}</div>
  {/if}

  {#if loading}
    <p class="muted">Loading…</p>
  {:else}
    <ul class="list">
      {#each profiles as p (p)}
        <li class="row">
          <div class="fields">
            <input
              class="name"
              bind:value={renameDraft[p]}
              disabled={busy !== ""}
              aria-label="Profile name for {p}"
            />
            <span class="id muted">{p === "default" ? "default" : p}</span>
          </div>
          <div class="actions">
            <button
              type="button"
              class="ghost"
              disabled={busy !== "" || (renameDraft[p] ?? "") === p}
              onclick={() => rename(p)}
            >
              Rename
            </button>
            <button type="button" class="primary" disabled={busy !== ""} onclick={() => open(p)}>
              Open
            </button>
            <button
              type="button"
              class="danger"
              disabled={busy !== "" || profiles.length <= 1}
              onclick={() => remove(p)}
            >
              Delete
            </button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}

  <section class="add">
    <h2>Add account</h2>
    <div class="add-row">
      <input
        placeholder="name (e.g. work)"
        bind:value={newName}
        disabled={busy !== ""}
        onkeydown={(e) => e.key === "Enter" && addNamed()}
      />
      <button type="button" class="primary" disabled={busy !== "" || !newName.trim()} onclick={addNamed}>
        Create
      </button>
      <button type="button" class="ghost" disabled={busy !== ""} onclick={addNext}>
        Quick add
      </button>
    </div>
  </section>
</main>

<style>
  :global(html, body) {
    margin: 0;
    height: 100%;
    background: #0b141a;
    color: #e9edef;
    font-family: system-ui, -apple-system, "Segoe UI", sans-serif;
  }

  main {
    box-sizing: border-box;
    min-height: 100%;
    padding: 1.25rem 1.35rem 1.75rem;
    max-width: 560px;
    margin: 0 auto;
  }

  header h1 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .sub {
    margin: 0.35rem 0 1rem;
    color: #8696a0;
    font-size: 0.9rem;
    line-height: 1.4;
  }

  .muted {
    color: #8696a0;
    font-size: 0.85rem;
  }

  .err {
    background: #3a1515;
    border: 1px solid #7f2a2a;
    color: #ffb4b4;
    padding: 0.65rem 0.75rem;
    border-radius: 8px;
    margin-bottom: 0.85rem;
    font-size: 0.88rem;
    word-break: break-word;
  }

  .list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
  }

  .row {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    padding: 0.75rem;
    background: #111b21;
    border: 1px solid #1f2c34;
    border-radius: 10px;
  }

  .fields {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .id {
    font-size: 0.75rem;
  }

  input {
    background: #0b141a;
    border: 1px solid #2a3942;
    color: #e9edef;
    border-radius: 8px;
    padding: 0.5rem 0.65rem;
    font-size: 0.95rem;
  }

  input:focus {
    outline: 1px solid #00a884;
    border-color: #00a884;
  }

  input:disabled {
    opacity: 0.6;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
  }

  button {
    border: none;
    border-radius: 8px;
    padding: 0.4rem 0.7rem;
    font-size: 0.85rem;
    cursor: pointer;
    font-weight: 500;
  }

  button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .primary {
    background: #00a884;
    color: #04281f;
  }

  .ghost {
    background: #1f2c34;
    color: #e9edef;
  }

  .danger {
    background: #3a1515;
    color: #ffb4b4;
  }

  .add {
    margin-top: 1.35rem;
    padding-top: 1rem;
    border-top: 1px solid #1f2c34;
  }

  .add h2 {
    margin: 0 0 0.65rem;
    font-size: 1rem;
    font-weight: 600;
  }

  .add-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
  }

  .add-row input {
    flex: 1 1 140px;
  }
</style>

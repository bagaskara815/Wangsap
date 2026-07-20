<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  async function openSettings() {
    try {
      await invoke("open_settings");
    } catch {
      window.location.href = "/settings";
    }
  }
</script>

<main class="shell">
  <h1>Wangsap</h1>
  <p>com.wangsap — WhatsApp Web for Linux</p>
  <button type="button" onclick={openSettings}>Manage accounts</button>
</main>

<style>
  :global(html, body) {
    margin: 0;
    height: 100%;
    background: #0b141a;
    color: #e9edef;
    font-family: system-ui, sans-serif;
  }
  .shell {
    min-height: 100%;
    display: grid;
    place-content: center;
    gap: 0.65rem;
    text-align: center;
  }
  h1 {
    margin: 0;
    font-weight: 600;
  }
  p {
    margin: 0;
    opacity: 0.65;
  }
  button {
    justify-self: center;
    margin-top: 0.5rem;
    background: #00a884;
    color: #04281f;
    border: none;
    border-radius: 8px;
    padding: 0.5rem 0.9rem;
    font-weight: 600;
    cursor: pointer;
  }
</style>

<script lang="ts">
  export let status = '';
</script>

<div class="splash">
  <div class="splash-logo">
    <div class="logo-emblem">✦</div>
    <div class="logo-title">Beauty Album</div>
    <div class="logo-sub">Black Desert Online</div>
  </div>

  <div class="splash-footer">
    <div class="bar-track">
      <div class="bar-fill"></div>
    </div>
    <span class="status-text">{status || 'Loading…'}</span>
  </div>
</div>

<style lang="scss">
  .splash {
    position: fixed;
    inset: 0;
    background: #0a0a0c;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    user-select: none;
  }

  .splash-logo {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    margin-bottom: 80px;
  }

  .logo-emblem {
    font-size: 40px;
    color: #c8a96e;
    opacity: 0.9;
    animation: pulse 2.4s ease-in-out infinite;
  }

  .logo-title {
    font-family: 'Courier New', monospace;
    font-size: 28px;
    font-weight: 700;
    letter-spacing: 0.18em;
    text-transform: uppercase;
    color: #e8d5b0;
  }

  .logo-sub {
    font-family: 'Courier New', monospace;
    font-size: 11px;
    letter-spacing: 0.28em;
    text-transform: uppercase;
    color: #5a5040;
  }

  .splash-footer {
    position: absolute;
    bottom: 48px;
    left: 50%;
    transform: translateX(-50%);
    width: 340px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
  }

  .bar-track {
    width: 100%;
    height: 2px;
    background: #1e1c18;
    border-radius: 1px;
    overflow: hidden;
    position: relative;
  }

  .bar-fill {
    position: absolute;
    top: 0;
    left: -60%;
    width: 60%;
    height: 100%;
    background: linear-gradient(90deg, transparent, #c8a96e 50%, transparent);
    animation: sweep 1.6s ease-in-out infinite;
  }

  .status-text {
    font-family: 'Courier New', monospace;
    font-size: 10px;
    letter-spacing: 0.2em;
    text-transform: uppercase;
    color: #3d3828;
  }

  @keyframes sweep {
    0%   { left: -60%; }
    100% { left: 110%; }
  }

  @keyframes pulse {
    0%, 100% { opacity: 0.6; transform: scale(1); }
    50%       { opacity: 1;   transform: scale(1.08); }
  }
</style>

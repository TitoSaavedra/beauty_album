<div class="skeleton-card">
  <div class="skeleton-thumb">
    <div class="shimmer"></div>
  </div>
  <div class="skeleton-footer">
    <div class="skeleton-line w60"></div>
    <div class="skeleton-line w40"></div>
  </div>
</div>

<style>
  .skeleton-card {
    background: #0f141a;
    border: 1px solid #1a232c;
    border-radius: 8px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .skeleton-thumb {
    aspect-ratio: 4 / 5;
    background: #0c1017;
    position: relative;
    overflow: hidden;
  }

  .shimmer {
    position: absolute;
    inset: 0;
    background: linear-gradient(
      105deg,
      transparent 35%,
      rgba(255, 204, 77, 0.045) 50%,
      transparent 65%
    );
    background-size: 200% 100%;
    animation: shimmer 1.6s ease-in-out infinite;
  }

  @keyframes shimmer {
    0%   { background-position: 200% 0; }
    100% { background-position: -200% 0; }
  }

  .skeleton-footer {
    padding: 10px 14px;
    background: #0c1015;
    border-top: 1px solid #141b23;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
  }

  .skeleton-line {
    height: 8px;
    border-radius: 4px;
    background: #1a232c;
    animation: pulse 1.6s ease-in-out infinite;
  }

  .w60 { width: 60%; }
  .w40 { width: 30%; }

  @keyframes pulse {
    0%, 100% { opacity: 0.6; }
    50%       { opacity: 0.25; }
  }
</style>

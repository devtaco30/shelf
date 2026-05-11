<script lang="ts">
  // value: "HH:MM" 24시간 형식 또는 빈 문자열
  export let value = '';

  let ampm = 'AM';
  let hour = 12;
  let minute = 0;

  $: parseExternal(value);

  function parseExternal(v: string) {
    if (!v) return;
    const [hStr, mStr] = v.split(':');
    const h = parseInt(hStr, 10);
    const m = parseInt(mStr, 10);
    const newAmpm = h >= 12 ? 'PM' : 'AM';
    const newHour = h % 12 || 12;
    const newMinute = m;
    if (ampm !== newAmpm || hour !== newHour || minute !== newMinute) {
      ampm = newAmpm;
      hour = newHour;
      minute = newMinute;
    }
  }

  function emit() {
    let h24 = hour % 12;
    if (ampm === 'PM') h24 += 12;
    value = `${String(h24).padStart(2, '0')}:${String(minute).padStart(2, '0')}`;
  }

  const hours = Array.from({ length: 12 }, (_, i) => i + 1);
  const minutes = Array.from({ length: 12 }, (_, i) => i * 5);
</script>

<div class="time-input">
  <select bind:value={ampm} on:change={emit}>
    <option value="AM">오전</option>
    <option value="PM">오후</option>
  </select>
  <select bind:value={hour} on:change={emit}>
    {#each hours as h}
      <option value={h}>{h}시</option>
    {/each}
  </select>
  <select bind:value={minute} on:change={emit}>
    {#each minutes as m}
      <option value={m}>{String(m).padStart(2, '0')}분</option>
    {/each}
  </select>
</div>

<style>
  .time-input {
    display: flex;
    gap: 4px;
  }
  select {
    padding: 5px 4px;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 13px;
    background: white;
    cursor: pointer;
  }
  select:first-child { flex: 0 0 auto; }
</style>

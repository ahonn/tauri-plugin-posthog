<script>
  import Greet from './lib/Greet.svelte'
  import { ping, PostHog, capture, identify, reset } from 'tauri-plugin-posthog-api'

	let response = $state('')
	let userName = $state('user-123')
	let eventName = $state('button_clicked')

	function updateResponse(returnValue) {
		response += `[${new Date().toLocaleTimeString()}] ` + (typeof returnValue === 'string' ? returnValue : JSON.stringify(returnValue)) + '<br>'
	}

	function _ping() {
		ping("Pong!").then(updateResponse).catch(updateResponse)
	}

	async function _captureEvent() {
		try {
			await PostHog.capture(eventName, {
				button: 'test-button',
				page: 'main',
				timestamp: new Date().toISOString()
			})
			updateResponse(`Event "${eventName}" captured successfully`)
		} catch (error) {
			updateResponse(`Error capturing event: ${error}`)
		}
	}

	async function _identify() {
		try {
			await PostHog.identify(userName, {
				email: `${userName}@example.com`,
				name: 'Test User',
				plan: 'free'
			})
			updateResponse(`User identified as: ${userName}`)
		} catch (error) {
			updateResponse(`Error identifying user: ${error}`)
		}
	}

	async function _getIds() {
		try {
			const distinctId = await PostHog.getDistinctId()
			const deviceId = await PostHog.getDeviceId()
			updateResponse(`Distinct ID: ${distinctId || 'none'}, Device ID: ${deviceId}`)
		} catch (error) {
			updateResponse(`Error getting IDs: ${error}`)
		}
	}

	async function _reset() {
		try {
			await PostHog.reset()
			updateResponse('User data reset successfully')
		} catch (error) {
			updateResponse(`Error resetting: ${error}`)
		}
	}

	async function _captureAnonymous() {
		try {
			await PostHog.captureAnonymous('anonymous_event', {
				action: 'test',
				anonymous: true
			})
			updateResponse('Anonymous event captured successfully')
		} catch (error) {
			updateResponse(`Error capturing anonymous event: ${error}`)
		}
	}

	async function _captureBatch() {
		try {
			await PostHog.captureBatch([
				{
					event: 'batch_event_1',
					properties: { batch: true, index: 1 }
				},
				{
					event: 'batch_event_2',
					properties: { batch: true, index: 2 }
				}
			])
			updateResponse('Batch events captured successfully')
		} catch (error) {
			updateResponse(`Error capturing batch events: ${error}`)
		}
	}
</script>

<main class="container">
  <h1>Welcome to Tauri!</h1>

  <div class="row">
    <a href="https://vite.dev" target="_blank">
      <img src="/vite.svg" class="logo vite" alt="Vite Logo" />
    </a>
    <a href="https://tauri.app" target="_blank">
      <img src="/tauri.svg" class="logo tauri" alt="Tauri Logo" />
    </a>
    <a href="https://svelte.dev" target="_blank">
      <img src="/svelte.svg" class="logo svelte" alt="Svelte Logo" />
    </a>
  </div>

  <p>
    Click on the Tauri, Vite, and Svelte logos to learn more.
  </p>

  <div class="row">
    <Greet />
  </div>

  <div class="posthog-testing">
    <h2>PostHog Plugin Testing</h2>
    
    <div class="input-section">
      <label>
        User Name:
        <input bind:value={userName} placeholder="user-123" />
      </label>
      <label>
        Event Name:
        <input bind:value={eventName} placeholder="button_clicked" />
      </label>
    </div>

    <div class="button-section">
      <button onclick="{_identify}">Identify User</button>
      <button onclick="{_captureEvent}">Capture Event</button>
      <button onclick="{_captureAnonymous}">Capture Anonymous</button>
      <button onclick="{_captureBatch}">Capture Batch</button>
      <button onclick="{_getIds}">Get IDs</button>
      <button onclick="{_reset}">Reset</button>
      <button onclick="{_ping}">Ping (Legacy)</button>
    </div>

    <div class="response-section">
      <h3>Response Log:</h3>
      <div class="response-log">{@html response}</div>
      <button onclick={() => response = ''}>Clear Log</button>
    </div>
  </div>

</main>

<style>
  .logo.vite:hover {
    filter: drop-shadow(0 0 2em #747bff);
  }

  .logo.svelte:hover {
    filter: drop-shadow(0 0 2em #ff3e00);
  }

  .posthog-testing {
    margin-top: 2rem;
    padding: 1rem;
    border: 1px solid #ccc;
    border-radius: 8px;
    background: #f9f9f9;
  }

  .input-section {
    margin-bottom: 1rem;
  }

  .input-section label {
    display: block;
    margin-bottom: 0.5rem;
  }

  .input-section input {
    width: 200px;
    padding: 0.5rem;
    margin-left: 0.5rem;
  }

  .button-section {
    margin-bottom: 1rem;
  }

  .button-section button {
    margin: 0.25rem;
    padding: 0.5rem 1rem;
    background: #0070f3;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .button-section button:hover {
    background: #0051a5;
  }

  .response-section {
    margin-top: 1rem;
  }

  .response-log {
    max-height: 300px;
    overflow-y: auto;
    background: #fff;
    border: 1px solid #ddd;
    padding: 1rem;
    margin: 0.5rem 0;
    font-family: monospace;
    font-size: 0.9rem;
  }
</style>

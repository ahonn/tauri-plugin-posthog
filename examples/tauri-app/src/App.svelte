<script>
  import { PostHog } from 'tauri-plugin-posthog-api'

	let response = $state('')
	let userName = $state('user-123')
	let eventName = $state('button_clicked')

	function updateResponse(returnValue) {
		response += `[${new Date().toLocaleTimeString()}] ` + (typeof returnValue === 'string' ? returnValue : JSON.stringify(returnValue)) + '<br>'
	}


	async function _captureEvent() {
		try {
			console.log('Attempting to capture event:', eventName)
			await PostHog.capture(eventName, {
				button: 'test-button',
				page: 'main',
				timestamp: new Date().toISOString()
			})
			updateResponse(`Event "${eventName}" captured successfully`)
		} catch (error) {
			console.error('Capture error:', error)
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
			updateResponse(`Distinct ID: ${distinctId || 'none'}`)
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

	async function _testFeatureFlag() {
		try {
			const isEnabled = await PostHog.isFeatureEnabled('test-feature')
			const flagValue = await PostHog.getFeatureFlag('test-feature')
			updateResponse(`Feature flag 'test-feature' - enabled: ${isEnabled}, value: ${flagValue}`)
		} catch (error) {
			updateResponse(`Error checking feature flag: ${error}`)
		}
	}

	async function _setPersonProperties() {
		try {
			await PostHog.setPersonProperties({
				subscription: 'pro',
				last_login: new Date().toISOString()
			})
			updateResponse('Person properties set successfully')
		} catch (error) {
			updateResponse(`Error setting person properties: ${error}`)
		}
	}

	async function _createAlias() {
		try {
			const aliasName = `alias_${userName}_${Date.now()}`
			await PostHog.alias(aliasName)
			updateResponse(`Alias created: ${aliasName}`)
		} catch (error) {
			updateResponse(`Error creating alias: ${error}`)
		}
	}
</script>

<main class="container">
  <div class="app">
    <h1>PostHog Plugin Test</h1>
    
    <div class="inputs">
      <input bind:value={userName} placeholder="user-123" />
      <input bind:value={eventName} placeholder="button_clicked" />
    </div>

    <div class="actions">
      <button onclick="{_captureEvent}">Capture</button>
      <button onclick="{_identify}">Identify</button>
      <button onclick="{_createAlias}">Alias</button>
      <button onclick="{_testFeatureFlag}">Feature Flag</button>
      <button onclick="{_setPersonProperties}">Set Properties</button>
      <button onclick="{_getIds}">Get IDs</button>
      <button onclick="{_reset}" class="danger">Reset</button>
    </div>

    <div class="log">
      <div class="log-content">{@html response}</div>
      <button onclick={() => response = ''} class="clear">Clear</button>
    </div>
  </div>

</main>

<style>
  .app {
    max-width: 600px;
    margin: 2rem auto;
    padding: 2rem;
    font-family: system-ui, sans-serif;
  }

  h1 {
    text-align: center;
    margin-bottom: 2rem;
    color: #333;
  }

  .inputs {
    display: flex;
    gap: 1rem;
    margin-bottom: 2rem;
  }

  .inputs input {
    flex: 1;
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 6px;
    font-size: 1rem;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    margin-bottom: 2rem;
  }

  .actions button {
    padding: 0.75rem 1rem;
    background: #007acc;
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.9rem;
  }

  .actions button:hover {
    background: #005999;
  }

  .actions button.danger {
    background: #dc3545;
  }

  .actions button.danger:hover {
    background: #c82333;
  }

  .log {
    position: relative;
  }

  .log-content {
    min-height: 200px;
    max-height: 400px;
    overflow-y: auto;
    background: #f8f9fa;
    border: 1px solid #dee2e6;
    border-radius: 6px;
    padding: 1rem;
    font-family: 'SF Mono', monospace;
    font-size: 0.85rem;
    line-height: 1.4;
    text-align: left;
  }

  .clear {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
    padding: 0.25rem 0.5rem;
    background: #6c757d;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.75rem;
  }

  .clear:hover {
    background: #5a6268;
  }
</style>

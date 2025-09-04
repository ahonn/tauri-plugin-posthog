import { invoke } from '@tauri-apps/api/core'
import posthog from 'posthog-js'

// Re-export types from PostHog JS SDK
export type { Properties, CaptureOptions } from 'posthog-js'

// Define GroupObject type locally since it's not exported from posthog-js
export interface GroupObject {
  [groupType: string]: string | number
}

// Configuration interface returned by Tauri backend
interface PostHogConfig {
  apiKey: string
  apiHost: string
  options?: {
    disableCookie?: boolean
    disableSessionRecording?: boolean
    capturePageview?: boolean
    capturePageleave?: boolean
    debug?: boolean
    persistence?: 'localStorage' | 'cookie' | 'memory' | 'localStorage+cookie' | 'sessionStorage'
    personProfiles?: 'always' | 'never' | 'identified_only'
  }
}

/**
 * PostHog client for Tauri applications
 * Wraps PostHog JS SDK with automatic configuration from Tauri backend
 */
export class PostHogTauri {
  private static initialized = false
  private static initPromise: Promise<void> | null = null

  /**
   * Initialize PostHog with configuration from Tauri backend
   * This is called automatically on first use
   */
  private static async init(): Promise<void> {
    if (this.initialized) return
    if (this.initPromise) return this.initPromise

    this.initPromise = this._performInit()
    return this.initPromise
  }

  private static async _performInit(): Promise<void> {
    try {
      // Get configuration from Tauri backend
      const config: PostHogConfig = await invoke('plugin:posthog|get_config')

      // Initialize PostHog JS SDK with backend configuration
      posthog.init(config.apiKey, {
        api_host: config.apiHost,
        ...config.options
      })

      this.initialized = true
    } catch (error) {
      console.error('Failed to initialize PostHog:', error)
      throw new Error(`PostHog initialization failed: ${error}`)
    }
  }

  /**
   * Capture an event with optional properties
   * @param event - The event name
   * @param properties - Event properties (optional)
   */
  static async capture(event: string, properties?: any): Promise<void> {
    await this.init()
    posthog.capture(event, properties)
  }

  /**
   * Identify a user with a distinct ID and optional properties
   * @param distinctId - The unique identifier for the user
   * @param properties - User properties (optional)
   */
  static async identify(distinctId: string, properties?: any): Promise<void> {
    await this.init()
    posthog.identify(distinctId, properties)
  }

  /**
   * Create an alias for the current user
   * @param alias - The alias to create
   */
  static async alias(alias: string): Promise<void> {
    await this.init()
    posthog.alias(alias)
  }

  /**
   * Reset the current user (clears distinct ID and other user data)
   */
  static async reset(): Promise<void> {
    await this.init()
    posthog.reset()
  }

  /**
   * Get the current distinct ID
   */
  static async getDistinctId(): Promise<string | undefined> {
    await this.init()
    return posthog.get_distinct_id()
  }

  /**
   * Group identify - associate user with a group
   * @param groupType - The type of group (e.g., 'company', 'project')
   * @param groupKey - The key for the group
   * @param properties - Group properties (optional)
   */
  static async groupIdentify(groupType: string, groupKey: string, properties?: any): Promise<void> {
    await this.init()
    posthog.group(groupType, groupKey, properties)
  }

  /**
   * Set person properties
   * @param properties - Properties to set on the person
   */
  static async setPersonProperties(properties: any): Promise<void> {
    await this.init()
    posthog.setPersonProperties(properties)
  }

  /**
   * Feature flag methods
   */
  static async isFeatureEnabled(flagKey: string): Promise<boolean> {
    await this.init()
    return posthog.isFeatureEnabled(flagKey) || false
  }

  static async getFeatureFlag(flagKey: string): Promise<string | boolean | undefined> {
    await this.init()
    return posthog.getFeatureFlag(flagKey)
  }

  static async getFeatureFlagPayload(flagKey: string): Promise<any> {
    await this.init()
    return posthog.getFeatureFlagPayload(flagKey)
  }

  static async reloadFeatureFlags(): Promise<void> {
    await this.init()
    posthog.reloadFeatureFlags()
  }

  /**
   * Page view tracking
   * @param properties - Page properties (optional)
   */
  static async capturePageView(properties?: any): Promise<void> {
    await this.init()
    posthog.capture('$pageview', properties)
  }

  /**
   * Get the PostHog JS SDK instance (advanced usage)
   * Ensures PostHog is initialized before returning
   */
  static async getInstance(): Promise<typeof posthog> {
    await this.init()
    return posthog
  }

  /**
   * Check if PostHog has been initialized
   */
  static isInitialized(): boolean {
    return this.initialized
  }

  /**
   * Manually initialize PostHog (optional - will be called automatically)
   */
  static async initialize(): Promise<void> {
    return this.init()
  }
}

// Default export
export default PostHogTauri

// Named exports
export { PostHogTauri as PostHog }

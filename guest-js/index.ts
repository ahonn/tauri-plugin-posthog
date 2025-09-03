import { invoke } from '@tauri-apps/api/core'

// Core interfaces matching PostHog JS SDK patterns
export interface Properties {
  [key: string]: any
}

export interface GroupObject {
  [groupType: string]: string | number
}

// Internal request interfaces for Tauri communication
interface CaptureRequest {
  event: string
  properties?: Properties
  distinctId?: string
  groups?: GroupObject
  timestamp?: string
  anonymous?: boolean
}

/**
 * PostHog client for Tauri applications
 * API designed to match PostHog JS SDK patterns
 */
export class PostHog {
  /**
   * Capture an event with optional properties
   * @param event - The event name
   * @param properties - Event properties (optional)
   */
  static async capture(event: string, properties?: Properties): Promise<void> {
    await invoke('plugin:posthog|capture', {
      request: {
        event,
        properties
      } as CaptureRequest
    })
  }

  /**
   * Identify a user with a distinct ID and optional properties
   * @param distinctId - The unique identifier for the user
   * @param properties - User properties (optional)
   */
  static async identify(distinctId: string, properties?: Properties): Promise<void> {
    await invoke('plugin:posthog|identify', {
      request: {
        distinctId,
        properties
      }
    })
  }

  /**
   * Create an alias for the current user
   * @param alias - The alias to create
   */
  static async alias(alias: string): Promise<void> {
    const distinctId = await this.getDistinctId()
    if (!distinctId) {
      throw new Error('Cannot create alias without a distinct ID. Call identify() first.')
    }
    
    await invoke('plugin:posthog|alias', {
      request: {
        distinctId,
        alias
      }
    })
  }

  /**
   * Reset the current user (clears distinct ID and other user data)
   */
  static async reset(): Promise<void> {
    await invoke('plugin:posthog|reset')
  }

  /**
   * Get the current distinct ID
   */
  static async getDistinctId(): Promise<string | null> {
    return await invoke('plugin:posthog|get_distinct_id')
  }

  /**
   * Get the device ID
   */
  static async getDeviceId(): Promise<string> {
    return await invoke('plugin:posthog|get_device_id')
  }

  /**
   * Capture multiple events in batch
   * @param events - Array of events to capture
   */
  static async captureBatch(events: Array<{
    event: string
    properties?: Properties
    timestamp?: Date
  }>): Promise<void> {
    const formattedEvents = events.map(event => ({
      event: event.event,
      properties: event.properties,
      timestamp: event.timestamp?.toISOString()
    }))

    await invoke('plugin:posthog|capture_batch', {
      request: { events: formattedEvents }
    })
  }

  // Advanced methods for power users
  
  /**
   * Capture an anonymous event (does not affect user identification)
   * @param event - The event name
   * @param properties - Event properties (optional)
   */
  static async captureAnonymous(event: string, properties?: Properties): Promise<void> {
    await invoke('plugin:posthog|capture', {
      request: {
        event,
        properties,
        anonymous: true
      } as CaptureRequest
    })
  }

  /**
   * Capture event with timestamp (for historical events)
   * @param event - The event name
   * @param properties - Event properties (optional)
   * @param timestamp - Event timestamp
   */
  static async captureWithTimestamp(event: string, properties: Properties | undefined, timestamp: Date): Promise<void> {
    await invoke('plugin:posthog|capture', {
      request: {
        event,
        properties,
        timestamp: timestamp.toISOString()
      } as CaptureRequest
    })
  }

  /**
   * Capture event with groups
   * @param event - The event name
   * @param properties - Event properties (optional)
   * @param groups - Group associations
   */
  static async captureWithGroups(event: string, properties: Properties | undefined, groups: GroupObject): Promise<void> {
    await invoke('plugin:posthog|capture', {
      request: {
        event,
        properties,
        groups
      } as CaptureRequest
    })
  }
}

// Default export (matching PostHog JS SDK pattern)
export default PostHog

// Convenience exports for functional programming style
export const capture = PostHog.capture.bind(PostHog)
export const identify = PostHog.identify.bind(PostHog)
export const alias = PostHog.alias.bind(PostHog)
export const reset = PostHog.reset.bind(PostHog)

// Alias for PostHog class (common pattern)
export { PostHog as posthog }

// Legacy ping function for backward compatibility
export async function ping(value: string): Promise<string | null> {
  return await invoke<{value?: string}>('plugin:posthog|ping', {
    payload: {
      value,
    },
  }).then((r) => (r.value ? r.value : null));
}

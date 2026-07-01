export type ErrorCode =
  | "install-failed"
  | "download-failed"
  | "version-detection-failed"
  | "health-check-failed"
  | "network-error"
  | "integrity-error"
  | "platform-unsupported"
  | "dependency-missing"
  | "wire-failed"
  | "unwire-failed"
  | "index-failed";

export interface ErrorContext {
  message: string;
  cause?: unknown;
  remediation?: string;
  [key: string]: unknown;
}

export class ToolError extends Error {
  public readonly code: ErrorCode;
  public readonly toolId: string;
  public readonly context: ErrorContext;

  constructor(toolId: string, code: ErrorCode, context: ErrorContext) {
    const message = `[${toolId}] ${context.message}`;
    super(message);
    this.name = "ToolError";
    this.toolId = toolId;
    this.code = code;
    this.context = context;

    if (context.cause instanceof Error) {
      this.stack = `${this.stack}\nCaused by: ${context.cause.stack}`;
    }
  }

  toJSON() {
    return {
      name: this.name,
      toolId: this.toolId,
      code: this.code,
      message: this.message,
      context: this.context,
    };
  }
}

export class InstallError extends ToolError {
  constructor(toolId: string, context: Omit<ErrorContext, "message"> & { message?: string }) {
    super(toolId, "install-failed", {
      message: context.message ?? `Failed to install ${toolId}`,
      ...context,
    });
    this.name = "InstallError";
  }
}

export class DownloadError extends ToolError {
  constructor(toolId: string, url: string, context: Omit<ErrorContext, "message"> & { message?: string }) {
    super(toolId, "download-failed", {
      message: context.message ?? `Failed to download ${toolId} from ${url}`,
      url,
      ...context,
    });
    this.name = "DownloadError";
  }
}

export class VersionError extends ToolError {
  constructor(toolId: string, context: Omit<ErrorContext, "message"> & { message?: string }) {
    super(toolId, "version-detection-failed", {
      message: context.message ?? `Failed to detect ${toolId} version`,
      ...context,
    });
    this.name = "VersionError";
  }
}

export class HealthCheckError extends ToolError {
  constructor(toolId: string, context: Omit<ErrorContext, "message"> & { message?: string }) {
    super(toolId, "health-check-failed", {
      message: context.message ?? `Health check failed for ${toolId}`,
      ...context,
    });
    this.name = "HealthCheckError";
  }
}

export class NetworkError extends ToolError {
  constructor(toolId: string, context: Omit<ErrorContext, "message"> & { message?: string }) {
    super(toolId, "network-error", {
      message: context.message ?? `Network error while installing ${toolId}`,
      ...context,
    });
    this.name = "NetworkError";
  }
}

export class IntegrityError extends ToolError {
  constructor(toolId: string, context: Omit<ErrorContext, "message"> & { message?: string }) {
    super(toolId, "integrity-error", {
      message: context.message ?? `Integrity check failed for ${toolId}`,
      ...context,
    });
    this.name = "IntegrityError";
  }
}

export class PlatformError extends ToolError {
  constructor(toolId: string, platform: string, context: Omit<ErrorContext, "message"> & { message?: string }) {
    super(toolId, "platform-unsupported", {
      message: context.message ?? `Platform ${platform} is not supported for ${toolId}`,
      platform,
      ...context,
    });
    this.name = "PlatformError";
  }
}

export class IndexError extends ToolError {
  constructor(toolId: string, context: Omit<ErrorContext, "message"> & { message?: string }) {
    super(toolId, "index-failed", {
      message: context.message ?? `Failed to index project with ${toolId}`,
      ...context,
    });
    this.name = "IndexError";
  }
}

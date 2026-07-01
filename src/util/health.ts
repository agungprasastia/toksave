export interface HealthStatus {
  healthy: boolean;
  version: string | null;
  issues: HealthIssue[];
}

export interface HealthIssue {
  severity: "error" | "warning";
  message: string;
  remediation?: string;
}

export interface RepairResult {
  success: boolean;
  message: string;
  healthAfterRepair?: HealthStatus;
}

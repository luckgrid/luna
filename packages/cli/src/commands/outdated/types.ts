import type { GoOutdatedProbe } from "../../lib/go";
import type { ProtoPinsOutdatedReport } from "../../lib/proto";

/** On-disk shape (mirrors the gather snapshot). */
export type StoredOutdatedSnapshot = {
  protoReport: ProtoPinsOutdatedReport;
  bunOut: string;
  uvProjects: { root: string; dryRunOut: string }[];
  goModules: { root: string; goGetDryRunOut: string; probe?: GoOutdatedProbe }[];
};

export type OutdatedSnapshot = StoredOutdatedSnapshot;

export type OutdatedSummaryState = {
  failed: number;
  stProto: number;
  stBun: number;
  stUv: number;
  stGo: number;
};

export type OutdatedTierId = "proto" | "bun" | "uv" | "go";

export type OutdatedCheckSummaryMode = "full" | "rollup";

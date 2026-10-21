"use server";

import { revalidatePath } from "next/cache";
import { getAdminIdentity } from "@/lib/admin-gate";
import {
  approveDeviceFlow,
  issueKey,
  revokeKey,
} from "@/lib/hub-admin";

/**
 * Server actions for the hub admin console (U8).
 * Every action re-checks the Clerk admin gate — server actions are public
 * POST endpoints, so the layout gate alone is not sufficient.
 */

export interface IssueKeyState {
  status: "idle" | "ok" | "error";
  error?: string;
  /** Plaintext member key — returned by the daemon exactly once. */
  minted?: { id: number; key: string; agent_id: string; team_id: string };
}

export interface SimpleState {
  status: "idle" | "ok" | "error";
  message?: string;
}

const TEAM_ID_RE = /^[a-z0-9._-]{1,64}$/;

export async function issueKeyAction(
  _prev: IssueKeyState,
  formData: FormData,
): Promise<IssueKeyState> {
  if (!(await getAdminIdentity())) {
    return { status: "error", error: "forbidden — admin only" };
  }
  const agentId = String(formData.get("agent_id") ?? "").trim();
  const teamId = String(formData.get("team_id") ?? "").trim();
  const isAdmin = formData.get("is_admin") === "on";

  if (!agentId) {
    return { status: "error", error: "agent_id is required" };
  }
  if (!TEAM_ID_RE.test(teamId)) {
    return {
      status: "error",
      error: "invalid team_id — use 1-64 chars from [a-z0-9._-]",
    };
  }

  const res = await issueKey(agentId, teamId, isAdmin);
  if (!res.ok) {
    return { status: "error", error: res.error };
  }
  revalidatePath("/dashboard/keys");
  revalidatePath("/dashboard");
  return {
    status: "ok",
    minted: {
      id: res.data.id,
      key: res.data.key,
      agent_id: agentId,
      team_id: teamId,
    },
  };
}

export async function revokeKeyAction(id: number): Promise<SimpleState> {
  if (!(await getAdminIdentity())) {
    return { status: "error", message: "forbidden — admin only" };
  }
  const res = await revokeKey(id);
  if (!res.ok) {
    return { status: "error", message: res.error };
  }
  revalidatePath("/dashboard/keys");
  revalidatePath("/dashboard");
  return { status: "ok", message: `key #${id} revoked` };
}

export async function approveDeviceAction(
  _prev: SimpleState,
  formData: FormData,
): Promise<SimpleState> {
  if (!(await getAdminIdentity())) {
    return { status: "error", message: "forbidden — admin only" };
  }
  const code = String(formData.get("code") ?? "").trim();
  const teamId = String(formData.get("team_id") ?? "").trim();
  if (!code) {
    return { status: "error", message: "missing device code" };
  }
  if (!TEAM_ID_RE.test(teamId)) {
    return {
      status: "error",
      message: "invalid team_id — use 1-64 chars from [a-z0-9._-]",
    };
  }
  const res = await approveDeviceFlow(code, teamId);
  if (!res.ok) {
    return { status: "error", message: res.error };
  }
  revalidatePath("/dashboard/approvals");
  revalidatePath("/dashboard");
  return {
    status: "ok",
    message: `${code} approved → team ${res.data.team_id ?? teamId}`,
  };
}

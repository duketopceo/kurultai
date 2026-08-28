import { createStore } from "/js/AlpineStore.js";
import * as API from "/js/api.js";

const BRAIN_ENDPOINT = "/plugins/kurultai/brain";

const model = {
  status: "checking", // online | offline | checking
  stats: null,
  error: null,
  loading: false,
  _initialized: false,

  init() {
    if (this._initialized) return;
    this._initialized = true;
    this.refresh();
  },

  async refresh() {
    if (this.loading) return;
    this.loading = true;
    try {
      const res = await API.callJsonApi(BRAIN_ENDPOINT, { action: "status" });
      if (res && res.success && res.ok !== false) {
        this.stats = res;
        this.status = "online";
        this.error = null;
      } else {
        this.status = "offline";
        this.error = (res && res.error) || "Brain unreachable";
      }
    } catch (e) {
      this.status = "offline";
      this.error = e?.message || "Status request failed";
    } finally {
      this.loading = false;
    }
  },

  fmtNum(n) {
    const v = Number(n);
    return Number.isFinite(v) ? v.toLocaleString() : "\u2014";
  },
};

export const store = createStore("kurultaiBrain", model);

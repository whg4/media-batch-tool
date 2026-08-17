import { createRouter, createWebHistory } from "vue-router";

export const ROUTES = {
  home: "/",
  drop: "/drop",
  template: "/template",
  processing: "/processing",
  done: "/done",
} as const;

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: ROUTES.home, name: "home", component: () => import("../views/HomeView.vue") },
    { path: ROUTES.drop, name: "drop", component: () => import("../views/DropView.vue") },
    { path: ROUTES.template, name: "template", component: () => import("../views/TemplateView.vue") },
    { path: ROUTES.processing, name: "processing", component: () => import("../views/ProcessingView.vue") },
    { path: ROUTES.done, name: "done", component: () => import("../views/DoneView.vue") },
  ],
});

export default router;

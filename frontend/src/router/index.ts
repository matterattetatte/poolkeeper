import { createRouter, createWebHistory } from "vue-router";
import HomeView from "../views/HomeView.vue";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: "/",
      name: "home",
      component: HomeView,
    },
    {
      path: "/borrow",
      name: "borrow",
      // route level code-splitting
      // this generates a separate chunk (About.[hash].js) for this route
      // which is lazy-loaded when the route is visited.
      component: () => import("../views/BorrowView.vue"),
    },
    {
      path: "/learn-more",
      name: "learn-more",
      // route level code-splitting
      // this generates a separate chunk (About.[hash].js) for this route
      // which is lazy-loaded when the route is visited.
      component: () => import("../views/LearnMoreView.vue"),
    },
    {
      path: "/about",
      name: "about",
      // route level code-splitting
      // this generates a separate chunk (About.[hash].js) for this route
      // which is lazy-loaded when the route is visited.
      component: () => import("../views/AboutView.vue"),
    },
    {
      path: '/lps',
      name: 'lps',
      component: () => import('../views/LiquidityPoolsView.vue')
    },
    {
      path: '/lps/:id',
      name: 'lp-details',
      component: () => import('../views/LiquidityPoolDetailsView.vue'),
      props: true
    }
  ],
});

export default router;

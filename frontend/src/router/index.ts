import { createRouter, createWebHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import MiniAppHome from '../views/MiniAppHome.vue'

const router = createRouter({
	history: createWebHistory(import.meta.env.BASE_URL),
	routes: [
		{
			path: '/',
			name: 'home',
			component: HomeView,
		},
		{
			path: '/about',
			name: 'about',
			component: () => import('../views/AboutView.vue'),
		},
		{
			path: '/learn',
			name: 'learn',
			component: () => import('../views/LearnMoreView.vue'),
		},
		{
			path: '/borrow',
			name: 'borrow',
			component: () => import('../views/BorrowView.vue'),
		},
		{
			path: '/pools',
			name: 'pools',
			component: () => import('../views/LiquidityPoolsView.vue'),
		},
		{
			path: '/pool/:id',
			name: 'pool-details',
			component: () => import('../views/LiquidityPoolDetailsView.vue'),
		},
		{
			path: '/miniapp',
			name: 'miniapp',
			component: MiniAppHome,
		},
	],
})

export default router

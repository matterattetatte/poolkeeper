<script setup lang="ts">
import { RouterView } from "vue-router";
import { RouterLink } from "vue-router";
import Logo from '@/components/Logo.vue';
import ConnectWallet from '@/components/ConnectWallet.vue'
import Footer from '@/components/Footer.vue';

</script>

<template>
   <nav class="flex items-center justify-start">
    <Logo />
    <RouterLink to="/" exact>Home</RouterLink>
    <RouterLink to="/pools">Liquidity Pools</RouterLink>
    <RouterLink to="/borrow">Borrowing (coming soon)</RouterLink>
    <RouterLink to="/learn-more">Learn More</RouterLink>
    <RouterLink to="/about">About</RouterLink>
    <ConnectWallet />
   </nav>
   <br>
  <RouterView />
  <Footer />
</template>

<style scoped>
nav {
  width: 100%;
  font-size: 12px;
  text-align: center;
  /* margin-top: 2rem; */
}

nav a.router-link-exact-active {
  color: var(--color-text);
}

nav a.router-link-exact-active:hover {
  background-color: transparent;
}

nav a {
  display: inline-block;
  padding: 0 1rem;
  border-left: 1px solid var(--color-border);
}

nav a:first-of-type {
  border: 0;
}

@media (min-width: 1024px) {
  nav {
    display: flex;
    font-size: 1rem;

    padding: 1rem 0;
    margin-top: 1rem;
  }
}
</style>

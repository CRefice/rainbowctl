<template>
  <div id="app">
    <light-form />
  </div>
</template>

<script>
import LightForm from "./components/LightForm.vue";

export default {
  name: "app",
  components: {
    LightForm
  }
};
</script>

<style>
html,
body,
form {
  height: 100%;
}

#app {
  padding: 5%;
  height: 100%;
  font-family: "Avenir", Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
}
</style>

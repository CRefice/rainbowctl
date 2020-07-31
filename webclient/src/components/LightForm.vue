<template>
  <main class="d-flex flex-column justify-content-around align-items-center">
    <div class="d-flex flex-wrap justify-content-center align-items-center">
      <b-button
        class="m-2"
        size="lg"
        @click="submit('off')"
        variant="outline-secondary"
        >Off</b-button
      >
      <b-button
        class="m-2"
        size="lg"
        @click="submit('solid')"
        variant="outline-secondary"
        >Solid</b-button
      >
      <b-button
        class="m-2"
        size="lg"
        @click="submit('pulse')"
        variant="outline-secondary"
        >Pulse</b-button
      >
      <b-button
        class="m-2"
        size="lg"
        @click="submit('chase')"
        variant="outline-secondary"
        >Chase</b-button
      >
      <b-button
        class="m-2"
        size="lg"
        @click="submit('chaseloop')"
        variant="outline-secondary"
        >Chase Loop</b-button
      >
      <b-button
        class="m-2"
        size="lg"
        @click="submit('rainbow')"
        variant="outline-secondary"
        >Rainbow</b-button
      >
      <b-button
        class="m-2"
        size="lg"
        @click="submit('equalizer')"
        variant="outline-secondary"
        >Equalizer</b-button
      >
    </div>
    <color-picker
      v-bind="color"
      :mouse-scroll="true"
      @input="onInput"
    ></color-picker>
  </main>
</template>

<script>
import ColorPicker from "@radial-color-picker/vue-color-picker";
import axios from "axios";
var throttle = require("lodash.throttle");

export default {
  components: { ColorPicker },
  data() {
    return {
      pattern: "off",
      color: {
        hue: 0,
        saturation: 100,
        lightness: 50
      },
      options: []
    };
  },
  methods: {
    onInput(hue) {
      this.color.hue = hue;
      this.submitColor();
    },
    submitColor: throttle(function() {
      axios.post("/api/color", {
        hue: this.color.hue,
        saturation: this.color.saturation / 100,
        lightness: this.color.lightness / 100
      });
    }, 60),
    submit(pattern) {
      axios.post(`/api/pattern/${pattern}`);
      this.submitColor();
    }
  }
};
</script>

<style>
@import "~@radial-color-picker/vue-color-picker/dist/vue-color-picker.min.css";

main {
  height: 100%;
}
</style>

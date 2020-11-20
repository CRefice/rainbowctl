function submit(pattern) {
  axios.post(`api/pattern/${pattern}`);
}

function changeColor(color) {
  rgb = color.rgb;
  axios.post(`api/color`, {
    red: rgb.r / 255,
    green: rgb.g / 255,
    blue: rgb.b / 255,
  });
}

var colorPicker = new iro.ColorPicker("#color-picker", {
  color: "#f00",
});
colorPicker.on("color:change", changeColor);

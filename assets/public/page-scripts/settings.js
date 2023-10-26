const pfp_picker = document.getElementById("pfp")
const pfp_form = document.getElementById("pfp-form");
const banner_picker = document.getElementById("banner");
const banner_form = document.getElementById("banner-form");
const stylesheet_picker = document.getElementById("stylesheet");
const stylesheet_form = document.getElementById("css-form");

pfp_picker.addEventListener("change", pfp_form.submit);
banner_picker.addEventListener("change", banner_form.submit);
stylesheet_picker.addEventListener("change", stylesheet_form.submit);


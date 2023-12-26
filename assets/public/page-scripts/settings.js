function addOnSubmits() {
  const pfp_picker = document.getElementById("pfp");
  const pfp_form = document.getElementById("pfp-form");
  const banner_picker = document.getElementById("banner");
  const banner_form = document.getElementById("banner-form");
  const stylesheet_picker = document.getElementById("stylesheet");
  const stylesheet_form = document.getElementById("css-form");

  pfp_picker.addEventListener("change", function (_) {
    console.log("Submitting pfp form");
    pfp_form.submit();
  });
  banner_picker.addEventListener("change", function (_) {
    console.log("Submitting banner form");
    banner_form.submit();
  });
  stylesheet_picker.addEventListener("change", function (_) {
    console.log("Submitting stylesheet form");
    stylesheet_form.submit();
  });
}

document.addEventListener("DOMContentLoaded", function () {
  addOnSubmits();
});

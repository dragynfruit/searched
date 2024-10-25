document.addEventListener('keydown', function () {
	document.getElementById('search').focus();
});

document.getElementById('search').addEventListener('keyup', function (event) {
	document.getElementById('search-complete').hidden = false;
});

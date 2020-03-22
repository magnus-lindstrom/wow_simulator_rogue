function ToggleTalentValue(checkbox) {
    if (checkbox.checked) {
        document.getElementById(checkbox.id + '-value').disabled = false;
    }
    else {
        document.getElementById(checkbox.id + '-value').disabled = true;
    }
}
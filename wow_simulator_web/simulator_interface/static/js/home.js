//$(document).on('submit', '#config-form', function(e){
//    e.preventDefault();
//
//    $.ajax({
//        type: 'POST',
//        url: 'api/create_cofig_file',
//        data: {
//            values:$('#drop').val(),
//            csrfmiddlewaretoken:$('input[name=csrfmiddlewaretoken]').val(),
//        },
//        success: function(dropdown_menu_selection){
//            console.log(dropdown_menu_selection);
//        },
//        error: function(xhr, errmsg, err){
//            console.log(xhr.status + ': ' + xhr.responseText);
//        }
//    });
//});
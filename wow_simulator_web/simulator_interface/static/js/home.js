// AJAX for posting
function load_config() {
    console.log("need to prevent default behavior") // sanity check
    $.ajax({
        url : "create_post/", // the endpoint
        type : "POST", // http method
        data : { the_post : $('#post-text').val() }, // data sent with the post request

        // handle a successful response
        success : function(json) {
            console.log(json); // log the returned json to the console
            console.log("success"); // another sanity check
        },

        // handle a non-successful response
        error : function(xhr,errmsg,err) {
            $('#results').html("<div class='alert-box alert radius' data-alert>Oops! We have encountered an error: "+errmsg+
                " <a href='#' class='close'>&times;</a></div>"); // add the error to the dom
            console.log(xhr.status + ": " + xhr.responseText); // provide a bit more info about the error to the console
        }
    });
};

$( document ).ready(function() {
    $('#file_to_load').on('change',function(){
        //get the file name
        var fileName = $(this).val().replace('C:\\fakepath\\', "");;
        //replace the "Choose a file" label
        $(this).next('.custom-file-label').html(fileName);
    })
});




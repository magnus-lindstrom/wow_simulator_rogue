from django.contrib import messages
from django.shortcuts import render
from django.views.generic import TemplateView


class HomeView(TemplateView):
    template_name = 'home.html'

    def post(self, request, *args, **kwargs):
        head_item = request.POST.get('drop')
        messages.add_message(request, messages.SUCCESS, 'Form submitted correctly')

        response = {
            'items': head_item,
        }
        return render(request, self.template_name, context=response)

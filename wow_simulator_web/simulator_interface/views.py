from django.contrib import messages
from django.shortcuts import render
from django.views.generic import TemplateView


class HomeView(TemplateView):
    template_name = 'home.html'

    # def get(self, request, *args, **kwargs):
        # fetch config files
        # parse config
        # create dictionary {slot_type: [item1, item2, ...]}

    def post(self, request, *args, **kwargs):
        head_item = request.POST.get('drop')
        # todo: how to get ALL selected items and dropdown labels dinamically?
        messages.add_message(request, messages.SUCCESS, 'Form submitted correctly')

        response = {
            'items': head_item,
        }
        return render(request, self.template_name, context=response)

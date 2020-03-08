from django.contrib import messages
from django.shortcuts import render
from django.views.generic import TemplateView
import os
import yaml

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))
ARMOR_FILE = os.path.join(REPO_ROOT, 'db', 'armor.yaml')
ENCHANTS_FILE = os.path.join(REPO_ROOT, 'db', 'enchants.yaml')


class HomeView(TemplateView):
    template_name = 'home.html'

    def get(self, request, *args, **kwargs):
        dropdown_armor_items = self._parse_armor_file()
        context = {'armor': dropdown_armor_items}

        return render(request, self.template_name, context=context)

    def post(self, request, *args, **kwargs):
        head_item = request.POST.get('drop')
        # todo: how to get ALL selected items and dropdown labels dinamically?
        messages.add_message(request, messages.SUCCESS, 'Form submitted correctly')

        response = {
            'items': head_item,
        }
        return render(request, self.template_name, context=response)

    @staticmethod
    def _parse_armor_file():
        with open(ARMOR_FILE, 'r') as armor_file:
            content = yaml.load(armor_file, yaml.FullLoader)

        slots = {}
        for item in content:
            slot_name = content[item]['slot']
            item_name = content[item]['name']
            if slot_name in slots:
                slots[slot_name].append(item_name)
            else:
                slots[slot_name] = [item_name]
        return slots

    @staticmethod
    def _parse_enchants_file():
        pass

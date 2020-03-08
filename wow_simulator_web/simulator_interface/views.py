from django.contrib import messages
from django.http import HttpResponseRedirect
from django.shortcuts import render, redirect
from django.urls import reverse
from django.views.generic import TemplateView
import os
import yaml

REPO_ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.realpath(__file__))))
ARMOR_FILE = os.path.join(REPO_ROOT, 'db', 'armor.yaml')
ENCHANTS_FILE = os.path.join(REPO_ROOT, 'db', 'enchants.yaml')

CONFIG_FILE = os.path.join(REPO_ROOT, 'configs', 'eugenia_test.yaml')


class HomeView(TemplateView):
    template_name = 'home.html'

    def get(self, request, *args, **kwargs):
        dropdown_armor_items = self._parse_armor_file()
        enchants = self._parse_enchants_file()

        context = {'armor': dropdown_armor_items,
                   'file_content': enchants}

        return render(request, self.template_name, context=context)

    def post(self, request, *args, **kwargs):
        armor_values = request.POST.getlist('armor')
        armor_dict = {'items': {'armor_names': [armor_value for armor_value in armor_values if armor_value]}}

        with open(CONFIG_FILE, 'w') as config_file:
            yaml.dump(armor_dict, config_file)

        messages.add_message(request, messages.SUCCESS, f"Config file created at: {CONFIG_FILE}")
        return redirect('/')

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
        with open(ENCHANTS_FILE, 'r') as enchants_file:
            content = yaml.load(enchants_file, yaml.FullLoader)

        return content

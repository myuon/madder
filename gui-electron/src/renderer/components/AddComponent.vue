<template>
  <div>
    <vs-button @click='popupActive=true' vs-color="primary" vs-type="border" vs-icon="add">New</vs-button>

    <vs-popup class="holamundo" title="New Component" :active.sync="popupActive">
      <vs-select
        vs-label="component_type"
        v-model="component.component_type"
      >
        <vs-select-item
          :key="index"
          :vs-value="item.value"
          :vs-text="item.text"
          v-for="(item, index) in component_type_options"
        />
      </vs-select>

      <vs-input vs-label="start_time" v-model.number="component.start_time" tyep="number" />
      <vs-input vs-label="length" v-model.number="component.length" type="number" />

      <vs-button @click="openFileDialog" vs-color="success" vs-type="border">File: {{ this.entity === "" ? "empty" : this.entity }}</vs-button>
      <vs-button @click="submit" vs-color="primary" vs-type="filled">Submit</vs-button>
    </vs-popup>
  </div>
</template>

<script>
  import { Component } from '@/lib';

  export default {
    name: 'add-component',
    props: [ 'onSubmit' ],
    data () {
      return {
        popupActive: false,
        component: new Component(-1, 'Video', 0, 1000, {}, []),
        entity: "",
        component_type_options: [
          { text: '動画', value: 'Video' },
          { text: '画像', value: 'Image' },
          { text: '音声', value: 'Sound' },
        ],
      };
    },
    methods: {
      openFileDialog () {
        this.$electron.remote.dialog.showOpenDialog(null, {}, (paths) => {
          if (paths != null && paths.length > 0) {
            this.entity = paths[0];
          }
        });
      },
      submit () {
        this.$emit('submit-new-component', Object.assign({data_path: this.entity}, this.component));
        this.popupActive = false;
      },
    },
  }
</script>

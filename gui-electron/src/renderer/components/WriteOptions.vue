<template>
  <vs-popup class="holamundo" title="New Component" :active.sync="popupActive">
    <vs-input vs-label="FPS" vs-placeholder="30" v-model.number="fps" type="number" />
    <vs-input vs-label="length (sec)" vs-placeholder="10" v-model.number="length" type="number" />

    <vs-button @click="openFileDialog" vs-color="success" vs-type="border">File: {{ this.uri === '' ? 'empty' : this.uri }}</vs-button>
    <vs-button @click="submit" vs-color="primary" vs-type="filled">Submit</vs-button>
  </vs-popup>
</template>

<script>
  import { Component } from '@/lib';

  export default {
    name: 'write-options',
    props: [ 'onSubmit' ],
    data () {
      return {
        popupActive: false,
        uri: '',
        fps: 30,
        length: 10,
      };
    },
    methods: {
      active () {
        this.popupActive = true;
      },
      openFileDialog () {
        this.$electron.remote.dialog.showOpenDialog(null, {}, (paths) => {
          if (paths != null && paths.length > 0) {
            this.uri = paths[0];
          }
        });
      },
      submit () {
        this.$emit('submit-start-render', {
          uri: this.uri,
          fps: this.fps,
          length: this.length,
        });
        this.popupActive = false;
      },
    },
  }
</script>

<template>
  <div ref="app" id="app">
    <screen :fetchScreen="fetchScreen" :position="position"></screen>
    <vs-slider @change="changePosition" v-model="position" max=1000 />
    <timeline ref="timeline" :components="components" :selected="selected" v-on:select="selectById"></timeline>
    <add-component v-on:submit-new-component="createNewComponent"></add-component>
    <component-detail ref="componentDetail" :component="selectedComponent" v-on:change-attr="changeAttrOnSelected"></component-detail>
    <write-options ref="writeOptions" v-on:submit-start-render="startRender"></write-options>
  </div>
</template>

<script>
  import Screen from '@/components/Screen';
  import Timeline from '@/components/Timeline';
  import AddComponent from '@/components/AddComponent';
  import ComponentDetail from '@/components/ComponentDetail';
  import WriteOptions from '@/components/WriteOptions';
  import { Communicator, Request, Receiver, cast_as, Component } from '@/lib';
  import { ipcRenderer } from 'electron'

  export default {
    name: 'gui-madder',
    components: {
      Screen,
      Timeline,
      AddComponent,
      ComponentDetail,
      WriteOptions,
    },
    data () {
      return {
        comm: new Communicator(() => {
          this.updateComponents();
        }),
        components: new Map(),
        selected: null,
        position: 0,
      };
    },
    computed: {
      selectedComponent () {
        return this.components.get(this.selected);
      }
    },
    methods: {
      updateComponents () {
        this.comm.send(
          Request.Get('/component'),
          Receiver.receive(response => {
            const comps = JSON.parse(response);

            let cmap = new Map();
            comps.forEach(_comp => {
              let comp = cast_as(Component.fromObject(_comp), Component);
              cmap.set(comp.id, comp);
            });

            this.components = cmap;
          })
        );
      },
      createNewComponent (component, callback) {
        this.comm.send(
          Request.Create('/component', component),
          Receiver.receive(data => {
            this.updateComponents();

            if (callback != null) {
              callback(data);
            }
          })
        );
      },
      selectById (component_id) {
        this.selected = component_id;
      },
      changePosition (value) {
        this.position = value;
      },
      fetchScreen (position, callback) {
        this.comm.send(
          Request.Get(`/screen/${position * 100}`),
          Receiver.receive(callback),
        );
      },
      changeAttrOnSelected (key, value) {
        const current = this.components.get(this.selected);

        this.comm.send(
          Request.Update(`/component/${current.id}`, { [key]: value }),
          Receiver.discard()
        );

        // This is not enough to trigger update for timeline, but why?
        this.$set(this.components, this.selected, Object.assign({ [key]: value }, this.selectedComponent));

        this.$refs.timeline.$forceUpdate();
        this.$refs.componentDetail.$forceUpdate();
      },
      startRender (config) {
        this.comm.send(
          Request.Create("/write", config),
          Receiver.receiveUntil(data => {
            console.log(`rendering finished! ${data}`);
          }, data => data === "")
        );
      },
    },
    mounted () {
      // file read/write
      ipcRenderer.on("open-yaml", (event, arg) => {
        this.comm.send(
          Request.Update("/project/yaml", arg),
          Receiver.receive(data => {
            this.updateComponents();
          })
        );
      });

      ipcRenderer.on("request-save-yaml", (event, arg) => {
        this.comm.send(
          Request.Get("/project/yaml"),
          Receiver.receive(data => {
            ipcRenderer.send("response-save-yaml", data);
          })
        );
      });

      ipcRenderer.on("request-write-avi-file", (event, arg) => {
        this.$refs.writeOptions.active();
      });
    },
  }
</script>

<style>
  /* CSS */
</style>

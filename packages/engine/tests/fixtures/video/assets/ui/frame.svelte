<meta name="component" content="frame">
<meta name="prop:texture" content="number" data-doc="The video player's texture; 0 shows nothing.">

<img id="Frame" src={$texture} alt="" hidden={!$texture}>

<style>
#Frame { position: absolute; left: 640px; top: 180px; width: 640px; height: 360px; }
</style>

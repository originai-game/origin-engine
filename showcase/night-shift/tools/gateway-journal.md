# Gateway journal

Every origin gateway result, written the moment it returns: the operation,
its id and its URL. A wrong row is corrected by a later row, never rewritten.
The README's asset table is made from this. The rows down to the first
account_usage row were written by hand, their times taken from file times
and replies, good to a few minutes; tools/gen-image.sh writes its own.
Media URLs are `https://api.origingame.ai/v1/media/<media>/content.<ext>`.

| When (UTC) | Operation | Name | Id | Result |
|---|---|---|---|---|
| 2026-11-17 08:27 | generate_image | courier concept (two earlier calls of this prompt returned images whose URLs were not recorded) | generation c77071fd-e8df-45b3-900b-0d02dec995e1 | media aed1983bbf2de2ca758b416fc0dad1bf (png) |
| 2026-11-17 08:31 | generate_3d image (op ns-courier-image-v1) | courier mesh | task 01a1e8a1-954c-7722-9225-e0b556d729d4 | media 854cc7f0cd867f1e3634bdf24c9491c4 (glb), 30 Meshy credits |
| 2026-11-17 08:33 | generate_image | pedestrian A concept | | media 4a5cb3986d2a9d9a2d65ae447e38c441 (png) |
| 2026-11-17 08:33 | generate_image | pedestrian B concept | | media 087e4847222b860fc2e1d106ae860a75 (png) |
| 2026-11-17 08:33 | generate_image | pedestrian C concept | | media 4a3c17e84f5aae2b2cf8477b2c6610f4 (png) |
| 2026-11-17 08:33 | generate_image | hover-bike concept | | media 9fbaeb47c736af43ef460505ee540689 (png) |
| 2026-11-17 08:34 | generate_3d image (op ns-bike-image-v1) | hover-bike mesh | task 01a1e8a4-2246-75a5-8ecc-70c1a472e3a7 | media 12d79351b68200cb1ae96c72b2b87cf0 (glb), 30 credits |
| 2026-11-17 08:34 | generate_3d image (op ns-ped-a-image-v1) | pedestrian A mesh | task 01a1e8a4-1cd3-7262-be4a-a49eb52b3dfc | media ed26b5f2f81deeb492624e2c874a7f30 (glb), 30 credits |
| 2026-11-17 08:34 | generate_3d image (op ns-ped-b-image-v1) | pedestrian B mesh | task 01a1e8a4-1d01-745c-837c-d471477a5a33 | media 2874ea37245c3acd0261b9a80331082b (glb), 30 credits |
| 2026-11-17 08:34 | generate_3d image (op ns-ped-c-image-v1) | pedestrian C mesh | task 01a1e8a4-1d02-72a5-b745-a4c2cafba937 | media 8a47eabcece3e858ac12832babf9d7d0 (glb), 30 credits |
| 2026-11-17 08:48 | generate_3d rig (op ns-courier-rig-v1) | courier rig, 1.82 m | task 01a1e8b1-97f8-76d1-8eab-683478128d8b | media 330ba6bb2d9c7bc5bcf025090bf87cdb (glb) |
| 2026-11-17 08:48 | generate_3d rig (op ns-ped-a-rig-v1) | pedestrian A rig, 1.78 m | task 01a1e8b1-9805-7392-8245-4b477bd64287 | media 59a588996118624a864dd167cef43962 (glb) |
| 2026-11-17 08:48 | generate_3d rig (op ns-ped-b-rig-v1) | pedestrian B rig, 1.66 m | task 01a1e8b1-980c-741c-a353-8f05790007a0 | media 8d07144d1f663eef657660964775d9b1 (glb) |
| 2026-11-17 08:48 | generate_3d rig (op ns-ped-c-rig-v1) | pedestrian C rig, 1.74 m | task 01a1e8b1-9806-7287-9085-dbadae98e864 | media 3d7cdd8b942ac31a6118bdf1f4b10eb1 (glb) |
| 2026-11-17 08:52 | generate_3d animate (op ns-courier-anim-v1) | courier clips: 0 Idle, 2 Alert | task 01a1e8b5-42a2-732d-a2d8-7eb189e02cd4 | pending |
| 2026-11-17 08:52 | generate_3d animate (op ns-ped-a-anim-v1) | pedestrian A clips: 695 Walk with Umbrella (in place), 0 Idle | task 01a1e8b5-429a-727e-93b2-5424c180ddad | pending |
| 2026-11-17 08:52 | generate_3d animate (op ns-ped-b-anim-v1) | pedestrian B clips: 695, 0 | task 01a1e8b5-429e-716b-b5a8-221ad698df22 | pending |
| 2026-11-17 08:52 | generate_3d animate (op ns-ped-c-anim-v1) | pedestrian C clips: 695, 56 Stand and Chat | task 01a1e8b5-4325-753f-89d1-638bcc312fdb | pending |
| 2026-11-17 08:56 | generate_image ×4 in one batch | stall, block A, block B, gate concepts | none returned | all four calls aborted on the client's timeout: outcome unknown, no URL. Reconcile against account usage before generating these again, one call at a time |
| 2026-11-17 08:58 | account_usage (1 day) | reconcile the timed-out batch | | cannot tell: the key is shared with the other hero threads (58 gpt-image-2.6-sunburst requests in the day across all of them) |
| 2026-11-17 08:59 | generate_image ×2 | stall, block A concepts (second attempt) | none returned | both aborted at the MCP client's 60 s limit: outcome unknown. The limit is the client's, so no further image calls go through it |
| 2026-11-17 09:01 | generate_image (gpt-image-2.6-sunburst) | concept-gate | none | HTTP 200, no URL: outcome unknown, not retried (.og/gateway/concept-gate.20261117T090144.json) |
| 2026-11-17 09:01 | generate_image (gpt-image-2.6-sunburst) | concept-stall | none | HTTP 200, no URL: outcome unknown, not retried (.og/gateway/concept-stall.20261117T090144.json) |
| 2026-11-17 09:01 | generate_image (gpt-image-2.6-sunburst) | concept-block-a | none | HTTP 200, no URL: outcome unknown, not retried (.og/gateway/concept-block-a.20261117T090144.json) |
| 2026-11-17 09:01 | generate_image (gpt-image-2.6-sunburst) | concept-block-b | none | HTTP 200, no URL: outcome unknown, not retried (.og/gateway/concept-block-b.20261117T090144.json) |
| 2026-11-17 09:03 | correction | concept-stall | generation 3d43b403-8be5-4b9f-bdc0-9a54d7d23abd | the 09:01 row above is wrong: the call succeeded with the bytes in the response (the script read only URLs then); sha256 1b06d1742241455b07862b59e5d03912097701f2b47494c0bb13acf1a58b9672 |
| 2026-11-17 09:03 | correction | concept-block-a | generation 5d010d5b-f5f5-4112-a60f-a7f7d60c2e76 | the 09:01 row above is wrong: the call succeeded with the bytes in the response (the script read only URLs then); sha256 146aa6dd38838dcfcb2f8c905493de7e733eae8cc10a330e910728e8686772b7 |
| 2026-11-17 09:03 | correction | concept-block-b | generation 6fd9bbdd-cdba-43f2-a3ce-204a64714c57 | the 09:01 row above is wrong: the call succeeded with the bytes in the response (the script read only URLs then); sha256 6d8b4de0e8623fa88230ef9f63283e5267f427d12ec08cf4aed8227b0fcf8529 |
| 2026-11-17 09:03 | correction | concept-gate | generation f5ca3c42-055a-48b4-97c2-23537a0733bb | the 09:01 row above is wrong: the call succeeded with the bytes in the response (the script read only URLs then); sha256 a6c98dafdc2e55faa4c80e2d58c6b581c8b2c41f89f6bd6767ccf98e8f3dae81 |
| 2026-11-17 09:04 | generate_prepare_upload | concept-stall as an image-to-3D input | media 5600c0c6e12eaaa3e70007d30c1cbae9 | upload slot; usable once generate_complete_upload confirms it |
| 2026-11-17 09:04 | generate_prepare_upload | concept-block-a as an image-to-3D input | media 54228c8249a7958e0b7f5d750602eb21 | upload slot; usable once generate_complete_upload confirms it |
| 2026-11-17 09:04 | generate_prepare_upload | concept-block-b as an image-to-3D input | media 5284618590c641f5dc9db205ea214d5a | upload slot; usable once generate_complete_upload confirms it |
| 2026-11-17 09:04 | generate_prepare_upload | concept-gate as an image-to-3D input | media 556cc72c27ca45b11b85128c9eb4949d | upload slot; usable once generate_complete_upload confirms it |
| 2026-11-17 09:05 | generate_complete_upload | concept-stall | (the media above) | confirmed: usable as media, expires 2026-11-24 |
| 2026-11-17 09:05 | generate_complete_upload | concept-block-a | (the media above) | confirmed: usable as media, expires 2026-11-24 |
| 2026-11-17 09:05 | generate_complete_upload | concept-block-b | (the media above) | confirmed: usable as media, expires 2026-11-24 |
| 2026-11-17 09:05 | generate_complete_upload | concept-gate | (the media above) | confirmed: usable as media, expires 2026-11-24 |
| 2026-11-17 09:07 | generate_3d image (operation_id ns-stall-image-v1) | stall from its concept image, 200k,4k, PBR, GLB | meshy task 01a1e8c1-044b-75fb-9425-fc5522f61ed6 | submitted; pending |
| 2026-11-17 09:07 | generate_3d image (operation_id ns-block-a-image-v1) | block-a from its concept image, 150k,4k, PBR, GLB | meshy task 01a1e8c0-fb64-766f-aa3f-e8181822e0e2 | submitted; pending |
| 2026-11-17 09:07 | generate_3d image (operation_id ns-block-b-image-v1) | block-b from its concept image, 150k,4k, PBR, GLB | meshy task 01a1e8c0-fb64-70de-a213-8542215d4401 | submitted; pending |
| 2026-11-17 09:07 | generate_3d image (operation_id ns-gate-image-v1) | gate from its concept image, 60k,2k, PBR, GLB | meshy task 01a1e8c0-fe74-747a-8762-0b03b759f6ee | submitted; pending |
| 2026-11-17 09:07 | generate_get_tasks | courier-animated result | meshy task 01a1e8b5-42a2-732d-a2d8-7eb189e02cd4, media f141befd4743917bc7189a5381b3bd9a | SUCCESS; https://api.origingame.ai/v1/media/f141befd4743917bc7189a5381b3bd9a/content.glb |
| 2026-11-17 09:07 | generate_get_tasks | ped_a-animated result | meshy task 01a1e8b5-429a-727e-93b2-5424c180ddad, media 96ccf805e0cbdec88f3e3e2e0b4f63d1 | SUCCESS; https://api.origingame.ai/v1/media/96ccf805e0cbdec88f3e3e2e0b4f63d1/content.glb |
| 2026-11-17 09:07 | generate_get_tasks | ped_b-animated result | meshy task 01a1e8b5-429e-716b-b5a8-221ad698df22, media 338099433a44441dc33b5d0b7eddfe82 | SUCCESS; https://api.origingame.ai/v1/media/338099433a44441dc33b5d0b7eddfe82/content.glb |
| 2026-11-17 09:07 | generate_get_tasks | ped_c-animated result | meshy task 01a1e8b5-4325-753f-89d1-638bcc312fdb, media 0d13e30151f3a3d9621cf7d613665038 | SUCCESS; https://api.origingame.ai/v1/media/0d13e30151f3a3d9621cf7d613665038/content.glb |
| 2026-11-17 09:07 | generate_get_tasks | gate result | meshy task 01a1e8c0-fe74-747a-8762-0b03b759f6ee, media c6ab1f36ce6e451997e15bb5c898f60b | SUCCESS; https://api.origingame.ai/v1/media/c6ab1f36ce6e451997e15bb5c898f60b/content.glb |
| 2026-11-17 09:09 | generate_get_tasks | block-b result | meshy task 01a1e8c0-fb64-70de-a213-8542215d4401, media f741d812d035167ab3982df7363be3f8 | SUCCESS; https://api.origingame.ai/v1/media/f741d812d035167ab3982df7363be3f8/content.glb |
| 2026-11-17 09:16 | generate_get_tasks | stall result | meshy task 01a1e8c1-044b-75fb-9425-fc5522f61ed6, media bbd8e2e0ef0d9196c23944aeba121370 | SUCCESS; https://api.origingame.ai/v1/media/bbd8e2e0ef0d9196c23944aeba121370/content.glb |
| 2026-11-17 09:16 | generate_get_tasks | block-a result | meshy task 01a1e8c0-fb64-766f-aa3f-e8181822e0e2, media a53d76ff8b6b9ab0ac912a89aa7bb900 | SUCCESS; https://api.origingame.ai/v1/media/a53d76ff8b6b9ab0ac912a89aa7bb900/content.glb |
| 2026-11-17 09:24 | assets_fetch_plan + assets_download | Cobblestone 01 (Poly Haven, CC0), 2k: cobblestone_01 diff, arm, nor_gl, disp | catalog group grp_776009be378c7de71b, release catalog-20261107-kenney-particles-v1 | downloaded to assets/textures/; sha256 matches the plan |
| 2026-11-17 09:24 | assets_fetch_plan + assets_download | Asphalt (Poly Haven, CC0), 2k: asphalt_02 diff, arm, nor_gl | catalog group grp_e95ff67aa62cb9993d, release catalog-20261107-kenney-particles-v1 | downloaded to assets/textures/; sha256 matches the plan |
| 2026-11-17 09:24 | assets_fetch_plan + assets_download | Brushed Concrete (Poly Haven, CC0), 2k: brushed_concrete diff, arm, nor_gl | catalog group grp_1474aca64e69a44dc0, release catalog-20261107-kenney-particles-v1 | downloaded to assets/textures/; sha256 matches the plan |
| 2026-11-17 09:24 | assets_fetch_plan + assets_download | Snow (Poly Haven, CC0), 2k: snow_02 diff, arm, nor_gl | catalog group grp_871469ce77923060c1, release catalog-20261107-kenney-particles-v1 | downloaded to assets/textures/; sha256 matches the plan |
| 2026-11-17 09:46 | generate_image (gpt-image-2.6-sunburst) | sign-chevrons | generation 6678b61b-1193-45a5-b1e4-c682c04be28f | bytes in the response, no media URL; sha256 d72b13891c6b3fffdf75385076fce12b6166ad1fd8bc6bc8203c5464e4540407 |
| 2026-11-17 09:47 | generate_image (gpt-image-2.6-sunburst) | sign-ring | generation 313ac34c-3248-4e90-bb50-d2f16d97d891 | bytes in the response, no media URL; sha256 153c8c056e9bab49ef7baf33e4eb29c3dc6c117c795788e06d626bb6d234ef3b |
| 2026-11-17 09:47 | generate_image (gpt-image-2.6-sunburst) | sign-triangle | generation e9cd686b-0d54-4575-a733-a636c691b21b | bytes in the response, no media URL; sha256 1f95a584d3479af6e920775d78ab09a5c7ae4a4bc7a752c79f6f5d83c8ae9788 |
| 2026-11-17 09:48 | generate_image (gpt-image-2.6-sunburst) | sign-bowl | generation 3bb9d47b-967d-4303-8042-acdddcd094ba | bytes in the response, no media URL; sha256 36b21bf425d24e68c75153bbdb7bb6737413598e6a73beabdc0f426dc17e781a |
| 2026-11-17 09:49 | generate_image (gpt-image-2.6-sunburst) | tower-windows | generation 5b6d3372-fab9-4860-acbb-43f7af40de49 | bytes in the response, no media URL; sha256 cf38070a971aae4c2f325a1d898ae9179cfaa6da946c330a4bace4d3d1183958 |
| 2026-11-17 10:07 | generate_image (gpt-image-2.6-sunburst) | lantern | generation 97ce8116-1867-488c-9a98-ac89ca9cad64 | bytes in the response, no media URL; sha256 741cbaa0ef0bb210cb9f777e8f5776b76450c28c8be564d743687be67bdc48b1 |
| 2026-11-17 10:08 | generate_prepare_upload | concept-lantern as an image-to-3D input | media 3c9c4a02a42d249f4a9a65306f039900 | upload slot; usable once generate_complete_upload confirms it |
| 2026-11-17 10:09 | generate_complete_upload | concept-lantern | (the media above) | confirmed: usable as media, expires 2026-11-24 |
| 2026-11-17 10:09 | generate_3d image (operation_id ns-lantern-image-v1) | lantern from its concept image, 20k,2k, PBR, GLB | meshy task 01a1e8fb-2b8b-7133-8380-b70427d8eac8 | submitted; pending |
| 2026-11-17 10:11 | generate_get_tasks | lantern result | meshy task 01a1e8fb-2b8b-7133-8380-b70427d8eac8, media 9dca0e1a2229f220749aec466da72ee7 | SUCCESS; https://api.origingame.ai/v1/media/9dca0e1a2229f220749aec466da72ee7/content.glb |
| 2026-11-17 12:27 | note | Asphalt, Snow and the cobblestone displacement map | (the 09:24 rows above) | downloaded but not used by the game: removed from assets/textures/, not committed |
| 2026-11-17 12:40 | download (dl.polyhaven.org, not the gateway) | Cobblestone 01 and Brushed Concrete, 1k: diff, nor_gl, arm | the catalog groups of the 09:24 rows | Poly Haven's own 1k files of the same CC0 sets replace the 2k ones (the 1024 map rule); sha256 in assets.json |
| 2026-11-17 12:50 | showcase-assets pin + upload | 21 shipped files: 10 models, 6 Poly Haven maps, 4 signs, the tower windows | assets.json | in origin-engine-models (R2) and origin-engine-showcase (OVH); `verify` read all 21 back |
| 2026-11-17 12:58 | archive (origin-generated-assets) | the raw Meshy outputs (image, rig and animate tasks), their concept images, the sign and window images | meshy/model/ns-<asset>-<image, rig, anim>-v1/, scenes/night-shift/textures/ | put once: model.glb, concept.png and task.json a task |

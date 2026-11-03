export const operations = [
  "botany",
  "inspect_character_source",
  "open_character_panel",
  "inspect_character_editor",
  "set_character_handle_view",
  "preview_character_handle",
  "set_character_handle",
  "load_character_editor_source",
  "upgrade_character_source",
  "connect_character_source",
  "set_character_parameters",
  "regenerate_character",
  "save_character_configuration",
  "load_character_configuration",
  "publish_character",
  "publish_terrain_region",
  "inspect_mesh_collisions",
  "save_portable_save",
  "load_portable_save",
  "inspect_portable_save",
  "migrate_portable_save",
  "save_checkpoint",
  "load_checkpoint",
  "inspect_checkpoint",
  "plan_world_regions",
  "query_terrain",
  "query_landform",
  "publish_landform",
  "save_landform",
  "load_landform",
  "plan_terrain_coverage",
  "publish_terrain",
  "save_terrain",
  "load_terrain",
  "capture_planet_reference",
  "query_planet",
  "publish_planet",
  "save_planet",
  "load_planet",
  "query_world_space",
  "set_npc_capacity",
  "set_avatar_audio",
  "inspect_npc_presentation",
  "set_npc_audio_output",
  "inspect_npc_audio_output",
  "capture_npc_audio",
  "interact",
  "talk_npc",
  "choose_dialogue",
  "close_dialogue",
  "inspect_interactions",
  "inspect_npc_routines",
  "inspect_dialogue",
  "set_interactions",
  "set_multiplayer_input",
  "release_multiplayer_input",
  "inspect_multiplayer_controls",
  "select_multiplayer_target",
  "pick_multiplayer",
  "use_multiplayer_ability",
  "use_multiplayer_ability_at",
  "cancel_multiplayer_ability",
  "set_multiplayer_view",
  "capture_multiplayer",
  "start_multiplayer_host",
  "stop_multiplayer_host",
  "inspect_multiplayer_host",
  "connect_multiplayer",
  "reconnect_multiplayer",
  "disconnect_multiplayer",
  "inspect_multiplayer_client",
  "send_multiplayer_movement",
  "send_multiplayer_operation",
  "model_editor_mesh",
  "set_editor_transform",
  "inspect_editor_transform",
  "preview_editor_transform",
  "transform_editor",
  "set_editor_mesh_view",
  "select_editor_mesh_region",
  "create_editor_mesh",
  "select_editor_mesh",
  "pick_editor_mesh",
  "inspect_project",
  "browse_project_directory",
  "create_project_folder",
  "inspect_project_library",
  "use_project_library",
  "register_project",
  "pin_project",
  "forget_project",
  "set_project_favorite",
  "discover_projects",
  "create_project",
  "open_project",
  "new_project_scene",
  "open_project_scene",
  "set_project_entry",
  "create_scene_object",
  "duplicate_scene_object",
  "inspect_editor",
  "set_viewport_performance",
  "inspect_viewport_performance",
  "compile_mesh_lods",
  "load_mesh_lods",
  "set_mesh_lod_settings",
  "inspect_mesh_lods",
  "select_editor_at",
  "edit_editor",
  "save_editor",
  "load_editor",
  "capture_editor",
  "compare_images",
  "plan_mesh_sweep",
  "preview_mesh_material_finish",
  "inspect_mesh_material_domain",
  "preview_mesh_material_domain",
  "preview_mesh_surface",
  "preview_mesh_ring_loft",
  "preview_mesh_junction_union",
  "inspect_mesh_boundaries",
  "preview_mesh_junction",
  "edit_plant",
  "inspect_plants",
  "inspect_plant",
  "inspect_plant_sampling",
  "inspect_plant_connections",
  "preview_plant",
  "edit_mesh",
  "preview_mesh_edit",
  "inspect_meshes",
  "inspect_mesh_objects",
  "project_landmarks",
  "inspect_mesh_islands",
  "inspect_mesh_topology",
  "select_mesh_components",
  "inspect_mesh_selection",
  "inspect_mesh",
  "inspect_mesh_normals",
  "inspect_mesh_pose",
  "inspect",
  "inspect_execution",
  "set_player_view",
  "inspect_player_view",
  "set_planet_runtime",
  "load_planet_runtime",
  "inspect_planet_runtime",
  "look_player_view",
  "set_player_view_input",
  "set_execution_mode",
  "inspect_instances",
  "inspect_character_controllers",
  "query_ground_traversal",
  "inspect_world",
  "inspect_origins",
  "inspect_population",
  "inspect_templates",
  "query_spawn_support",
  "query_geometry",
  "inspect_generators",
  "control_npc_generator",
  "spawn_actor",
  "despawn_actor",
  "inspect_abilities",
  "inspect_attribute",
  "inspect_resources",
  "inspect_vitality",
  "activate_ability",
  "aim_ability",
  "aim_ability_at",
  "cancel_ability",
  "inspect_navigation",
  "inspect_npcs",
  "inspect_npc_mind",
  "inspect_npc_circulation",
  "inspect_npc_tactics",
  "inspect_npc_cooperation",
  "order_npc",
  "order_npc_group",
  "inspect_relations",
  "inspect_targeting",
  "inspect_knowledge",
  "inspect_hearing",
  "inspect_objectives",
  "inspect_territory",
  "inspect_passages",
  "inspect_mobility",
  "query_navigation_generation",
  "set_npc_generators",
  "set_npc_templates",
  "set_ability_catalog",
  "set_spatial_network",
  "set_navigation_generation",
  "set_passages",
  "set_faction_relations",
  "inspect_noises",
  "inspect_footsteps",
  "inspect_patrols",
  "inspect_triggers",
  "inspect_selection",
  "select_entity",
  "select_at",
  "set_debug_view",
  "emit_noise",
  "generate",
  "edit_scene",
  "edit_scene_live",
  "preview_scene_edits",
  "upsert",
  "remove",
  "set_camera",
  "set_light",
  "undo",
  "redo",
  "play",
  "pause",
  "reset",
  "set_input",
  "bind_control",
  "set_control_input",
  "release_control",
  "inspect_controls",
  "step",
  "events",
  "save_scenario",
  "inspect_scenario",
  "run_scenario",
  "save_scene",
  "load_scene",
  "save_replay",
  "run_replay",
  "inspect_replay",
  "capture_replay",
  "capture",
  "exit"
];
export const systems = [
  {
    "group": "world",
    "title": [
      "Scenes with an origin",
      "Cenas que guardam sua origem"
    ],
    "body": [
      "Versioned scenes preserve IDs, references, procedural sources and configuration. The seeded room recipe remains available alongside native terrain, plant and character documents. Projects collect scenes and an executable entry point.",
      "Cenas versionadas preservam IDs, referências, fontes procedurais e configuração. A receita seeded de sala continua disponível ao lado dos novos documentos nativos de terrenos, plantas e personagens. Projetos reúnem cenas e uma entrada executável."
    ],
    "limit": [
      "Each generator has its own contract; universal whole-game generation is not implemented.",
      "Cada gerador tem seu próprio contrato; não há geração universal de um jogo inteiro."
    ],
    "commands": [
      "generate",
      "save_scene",
      "load_scene"
    ],
    "roots": [
      "world",
      "tools"
    ],
    "id": "system-1",
    "sources": [
      "docs/projects.md"
    ],
    "features": [
      [
        "Shared typed control through MCP",
        "Inspectable execution and explicit failure",
        "Versioned authorship or bounded session state"
      ],
      [
        "Controle tipado compartilhado pelo MCP",
        "Execução inspecionável e falha explícita",
        "Autoria versionada ou estado de sessão limitado"
      ]
    ],
    "media": "editor"
  },
  {
    "group": "world",
    "title": [
      "Edit with a way back",
      "Editar sem perder o caminho de volta"
    ],
    "body": [
      "Typed batch edits, validation before commit, final-state reference checks, expected revisions and transactional undo/redo. Authored documents stay separate from simulation.",
      "Edições tipadas em lote, validação antes de aplicar, referências verificadas no estado final, revisão esperada e undo/redo em uma transação. Documento autoral e simulação permanecem separados."
    ],
    "limit": [
      "Undo reverses authorship, not physics time. Applied edits restart the paused simulation.",
      "Undo reverte autoria, não o tempo da física. Edições aplicadas reiniciam a simulação pausada."
    ],
    "commands": [
      "edit_scene",
      "upsert",
      "remove",
      "undo",
      "redo"
    ],
    "roots": [
      "world",
      "tools"
    ],
    "id": "system-2",
    "sources": [
      "docs/commands.md"
    ],
    "features": [
      [
        "Shared typed control through MCP",
        "Inspectable execution and explicit failure",
        "Versioned authorship or bounded session state"
      ],
      [
        "Controle tipado compartilhado pelo MCP",
        "Execução inspecionável e falha explícita",
        "Autoria versionada ou estado de sessão limitado"
      ]
    ],
    "media": "editor"
  },
  {
    "group": "world",
    "title": [
      "Physics in explicit steps",
      "Física em passos explícitos"
    ],
    "body": [
      "Rapier physics with gravity, collisions, fixed 60 Hz simulation and per-character input. Recorded execution preserves replay; continuous mode keeps gameplay advancing without an unbounded command history.",
      "Física Rapier com gravidade, colisões, simulação fixa a 60 Hz e entrada por personagem. Execução gravada preserva replay; modo contínuo mantém o jogo avançando sem acumular um histórico de comandos infinito."
    ],
    "limit": [
      "The fixed step is a simulation contract, not an FPS guarantee for every scene.",
      "O passo fixo é contrato da simulação, não uma garantia de FPS em qualquer cena."
    ],
    "commands": [
      "play",
      "pause",
      "reset",
      "set_input",
      "step"
    ],
    "roots": [
      "character"
    ],
    "id": "system-3",
    "sources": [
      "docs/continuous-runtime.md"
    ],
    "features": [
      [
        "Shared typed control through MCP",
        "Inspectable execution and explicit failure",
        "Versioned authorship or bounded session state"
      ],
      [
        "Controle tipado compartilhado pelo MCP",
        "Execução inspecionável e falha explícita",
        "Autoria versionada ou estado de sessão limitado"
      ]
    ],
    "media": "npc"
  },
  {
    "group": "world",
    "title": [
      "A visible, selectable world",
      "Um mundo visível e selecionável"
    ],
    "body": [
      "Native wgpu renderer with cameras, directional light, shadows, procedural materials, dielectric and metallic response, prefiltered environment and atmosphere. The editor selects objects and components; offscreen captures use the same pipeline.",
      "Renderer nativo wgpu com câmera, luz direcional, sombras, materiais procedurais, resposta dielétrica e metálica, ambiente pré-filtrado e atmosfera. O editor seleciona objetos e componentes; capturas offscreen usam o mesmo pipeline."
    ],
    "limit": [
      "Rendering is validated first on macOS/Metal. WebAssembly is not delivered; advanced transparency is in development.",
      "Renderização validada primeiro em macOS/Metal. WebAssembly ainda não foi entregue; transparência avançada está em desenvolvimento."
    ],
    "commands": [
      "set_camera",
      "set_light",
      "select_entity",
      "select_at",
      "inspect_selection",
      "capture"
    ],
    "roots": [
      "render",
      "tools"
    ],
    "id": "system-4",
    "sources": [
      "docs/procedural-environment.md"
    ],
    "features": [
      [
        "Shared typed control through MCP",
        "Inspectable execution and explicit failure",
        "Versioned authorship or bounded session state"
      ],
      [
        "Controle tipado compartilhado pelo MCP",
        "Execução inspecionável e falha explícita",
        "Autoria versionada ou estado de sessão limitado"
      ]
    ],
    "media": "horizon"
  },
  {
    "group": "world",
    "title": [
      "Volumes that detect passage",
      "Volumes que percebem passagem"
    ],
    "body": [
      "AABB triggers, explicit targets, post-physics entry and exit, and bounded event history. Visibility can change independently of collision or perception.",
      "Gatilhos AABB, alvos explícitos, entrada e saída após a física e histórico limitado de eventos. Visibilidade pode mudar sem alterar colisão ou percepção."
    ],
    "limit": [
      "Current volumes are fixed and unrotated; detection is discrete.",
      "Volumes atuais são fixos e sem rotação; detecção discreta."
    ],
    "commands": [
      "inspect_triggers",
      "events"
    ],
    "roots": [
      "world"
    ],
    "id": "system-5",
    "sources": [
      "docs/commands.md"
    ],
    "features": [
      [
        "Shared typed control through MCP",
        "Inspectable execution and explicit failure",
        "Versioned authorship or bounded session state"
      ],
      [
        "Controle tipado compartilhado pelo MCP",
        "Execução inspecionável e falha explícita",
        "Autoria versionada ou estado de sessão limitado"
      ]
    ]
  },
  {
    "group": "minds",
    "title": [
      "Seeing is not knowing everything",
      "Ver não é saber tudo"
    ],
    "body": [
      "Volumetric XYZ vision with yaw and pitch, range, field of view and occlusion. Gradual per-individual confirmation separates suspicion from recognition and memory.",
      "Visão volumétrica XYZ com yaw e pitch, alcance, campo visual e oclusão. Confirmação gradual por indivíduo separa suspeita de reconhecimento e memória."
    ],
    "limit": [
      "Sensor orientation does not animate the body. Omniscience is an explicit policy only.",
      "Orientação sensorial não anima o corpo. Onisciência só existe como política explícita."
    ],
    "commands": [
      "inspect_npcs",
      "inspect_knowledge",
      "set_debug_view"
    ],
    "roots": [
      "npcs"
    ],
    "id": "system-6",
    "sources": [
      "docs/commands.md"
    ],
    "features": [
      [
        "Shared typed control through MCP",
        "Inspectable execution and explicit failure",
        "Versioned authorship or bounded session state"
      ],
      [
        "Controle tipado compartilhado pelo MCP",
        "Execução inspecionável e falha explícita",
        "Autoria versionada ou estado de sessão limitado"
      ]
    ],
    "media": "npc"
  },
  {
    "group": "minds",
    "title": [
      "Hear a clue. Not a live coordinate.",
      "Ouvir uma pista. Não uma coordenada viva."
    ],
    "body": [
      "Anonymous acoustic clues preserve emission position and time, acoustic transmission and attention arbitration. Footsteps derive from physical motion and locomotion profile. Presentation has a separate spatial-audio and PCM mixing system.",
      "Pistas sonoras anônimas preservam a posição e o instante de emissão, transmissão acústica e arbitragem de atenção. Passos derivam do movimento físico e do perfil de locomoção. Apresentação possui um sistema separado de áudio espacial e mixagem PCM."
    ],
    "limit": [
      "Hearing does not track a hidden source. Audio-device output is optional and disabled by default.",
      "Conhecimento auditivo não acompanha uma fonte oculta. Saída de áudio é opcional e desligada por padrão."
    ],
    "commands": [
      "emit_noise",
      "inspect_hearing",
      "inspect_noises",
      "inspect_footsteps"
    ],
    "roots": [
      "npcs"
    ],
    "id": "system-7",
    "sources": [
      "docs/npc-presentation-complete.md"
    ],
    "features": [
      [
        "Shared typed control through MCP",
        "Inspectable execution and explicit failure",
        "Versioned authorship or bounded session state"
      ],
      [
        "Controle tipado compartilhado pelo MCP",
        "Execução inspecionável e falha explícita",
        "Autoria versionada ou estado de sessão limitado"
      ]
    ]
  },
  {
    "group": "minds",
    "title": [
      "Autonomy, relationships and targets",
      "Autonomia, relações e alvos"
    ],
    "body": [
      "NPCs can operate without targeting the player, perceive other NPCs and select observed hostiles. Factions, directional attitudes and hysteresis govern eligibility and switching.",
      "NPCs podem existir sem jogador como alvo, perceber outros NPCs e selecionar hostis observados. Facções, atitudes direcionais e histerese controlam elegibilidade e troca."
    ],
    "limit": [
      "Eligibility never supplies a hidden location.",
      "Elegibilidade nunca fornece a localização oculta de alguém."
    ],
    "commands": [
      "set_faction_relations",
      "inspect_relations",
      "inspect_targeting"
    ],
    "roots": [
      "npcs"
    ],
    "id": "system-8",
    "sources": [
      "docs/commands.md"
    ],
    "features": [
      [
        "Shared typed control through MCP",
        "Inspectable execution and explicit failure",
        "Versioned authorship or bounded session state"
      ],
      [
        "Controle tipado compartilhado pelo MCP",
        "Execução inspecionável e falha explícita",
        "Autoria versionada ou estado de sessão limitado"
      ]
    ]
  },
  {
    "group": "minds",
    "title": [
      "Memory, search and communication",
      "Memória, busca e comunicação"
    ],
    "body": [
      "Last known positions, expiry, search hypotheses and explicit budgets. Allies can share confirmed observations through channels, XYZ radius and preserved TTL.",
      "Última posição conhecida, expiração, hipóteses de busca e budgets explícitos. Aliados podem compartilhar observações confirmadas por canal, raio XYZ e TTL preservado."
    ],
    "limit": [
      "Reports do not cascade; a search hypothesis never becomes a real observation.",
      "Relatos não se propagam em cascata; hipótese de busca não vira observação real."
    ],
    "commands": [
      "inspect_knowledge",
      "inspect_hearing",
      "inspect_npcs"
    ],
    "roots": [
      "npcs"
    ],
    "id": "system-9",
    "sources": [
      "docs/commands.md"
    ],
    "features": [
      [
        "Shared typed control through MCP",
        "Inspectable execution and explicit failure",
        "Versioned authorship or bounded session state"
      ],
      [
        "Controle tipado compartilhado pelo MCP",
        "Execução inspecionável e falha explícita",
        "Autoria versionada ou estado de sessão limitado"
      ]
    ]
  },
  {
    "group": "minds",
    "title": [
      "Decisions that explain themselves",
      "Decisões que explicam seus motivos"
    ],
    "body": [
      "Priorities and conditions compose response, fleeing, guarding, waiting, patrol, tactics, interaction and abilities. Cognition and traits influence choices and effort; inspection exposes scores, reasons and next evaluation without implicit omniscience.",
      "Prioridades e condições compõem resposta, fuga, guarda, espera, patrulha, tática, interação e habilidades. Cognição e traços influenciam escolhas e esforço; inspeção expõe scores, motivos e próxima avaliação, sem acesso onisciente implícito."
    ],
    "limit": [
      "NPC v1 has technical closure; art, perceived behavior and load scenarios retain separate acceptance gates.",
      "O NPC v1 tem fechamento técnico; arte, comportamento humano percebido e cenários de escala mantêm seus próprios gates."
    ],
    "commands": [
      "inspect_objectives",
      "inspect_npcs"
    ],
    "roots": [
      "npcs"
    ],
    "id": "system-10",
    "sources": [
      "docs/npc-v1-closure.md"
    ],
    "features": [
      [
        "Shared typed control through MCP",
        "Inspectable execution and explicit failure",
        "Versioned authorship or bounded session state"
      ],
      [
        "Controle tipado compartilhado pelo MCP",
        "Execução inspecionável e falha explícita",
        "Autoria versionada ou estado de sessão limitado"
      ]
    ]
  },
  {
    "group": "minds",
    "title": [
      "Routines with a place in the world",
      "Rotinas com lugar no mundo"
    ],
    "body": [
      "XYZ patrols, loop/ping-pong, waiting and resumption coexist with cyclic schedules, resource-driven needs and points of interest. Territory constrains clue acquisition and physical return under explicit rules.",
      "Patrulhas XYZ, loop/ping-pong, pausas e retomada convivem com agendas cíclicas, necessidades representadas por recursos e pontos de interesse. Território mantém aquisição de pistas e retorno físico sob regras explícitas."
    ],
    "limit": [
      "Schedules are game-authored; circular territory is not geopolitics or a universal social simulation.",
      "Agendas são configuração do jogo; território circular não constitui geopolítica ou uma simulação social universal."
    ],
    "commands": [
      "inspect_patrols",
      "inspect_territory"
    ],
    "roots": [
      "npcs"
    ],
    "id": "system-11",
    "sources": [
      "docs/npc-routines-interactions.md"
    ],
    "features": [
      [
        "Shared typed control through MCP",
        "Inspectable execution and explicit failure",
        "Versioned authorship or bounded session state"
      ],
      [
        "Controle tipado compartilhado pelo MCP",
        "Execução inspecionável e falha explícita",
        "Autoria versionada ou estado de sessão limitado"
      ]
    ]
  },
  {
    "group": "minds",
    "title": [
      "Find a route. Share a passage.",
      "Encontrar caminho. Dividir passagem."
    ],
    "body": [
      "Planar and spatial navigation, partial routes, recovery, local connectors and exclusively reserved passages. Ground control queries real geometry for slopes and steps; circulation handles neighbors while preserving physical authority.",
      "Navegação planar e espacial, caminhos parciais, recuperação, conectores locais e passagens com reserva exclusiva. O controlador terrestre consulta geometria real para rampas/degraus; circulação trata vizinhos e preserva a autoridade física."
    ],
    "limit": [
      "Generated networks have declared regions and budgets. This is not a global navmesh or a universal deadlock-free guarantee.",
      "Redes geradas têm região e budgets declarados. Não é navmesh global nem prova de ausência universal de deadlocks."
    ],
    "commands": [
      "inspect_navigation",
      "set_passages",
      "inspect_passages"
    ],
    "roots": [
      "npcs"
    ],
    "id": "system-12",
    "sources": [
      "docs/ground-local-connections.md"
    ],
    "features": [
      [
        "Shared typed control through MCP",
        "Inspectable execution and explicit failure",
        "Versioned authorship or bounded session state"
      ],
      [
        "Controle tipado compartilhado pelo MCP",
        "Execução inspecionável e falha explícita",
        "Autoria versionada ou estado de sessão limitado"
      ]
    ]
  },
  {
    "group": "minds",
    "title": [
      "Movement in three dimensions",
      "Movimento que ocupa três dimensões"
    ],
    "body": [
      "XYZ network with walk/jump/glide/fly capabilities, local connections, composed support and partial approaches. Hybrid locomotion integrates ground KCC and aerial segments; curved flight controls acceleration, turning, hover and orbit.",
      "Rede XYZ com capacidades walk/jump/glide/fly, conexões locais, apoio composto e aproximação parcial. O motor híbrido integra KCC terrestre e trechos aéreos; voo curvado controla aceleração, giro, hover e órbita."
    ],
    "limit": [
      "Actor capability and physical viability are revalidated. No full aerodynamics or unbudgeted free-space navigation.",
      "A capacidade do ator e a viabilidade física são revalidadas. Sem aerodinâmica completa ou navegação livre sem orçamento."
    ],
    "commands": [
      "set_spatial_network",
      "inspect_mobility",
      "inspect_navigation"
    ],
    "roots": [
      "npcs",
      "character"
    ],
    "id": "system-13",
    "sources": [
      "docs/npc-hybrid-mobility.md"
    ],
    "features": [
      [
        "Shared typed control through MCP",
        "Inspectable execution and explicit failure",
        "Versioned authorship or bounded session state"
      ],
      [
        "Controle tipado compartilhado pelo MCP",
        "Execução inspecionável e falha explícita",
        "Autoria versionada ou estado de sessão limitado"
      ]
    ],
    "media": "npc"
  },
  {
    "group": "actions",
    "title": [
      "Abilities, one shared executor",
      "Habilidades, um executor compartilhado"
    ],
    "body": [
      "Player and NPC share one executor: scaled integer resources, atomic costs, casts, cooldowns and cancellation. Movement can remain free, interrupt or be locked by a cast.",
      "Jogador e NPC usam o mesmo executor: recursos inteiros com escala, custos atômicos, cast, cooldown e cancelamento. Movimento pode ser livre, interromper ou ser bloqueado pelo cast."
    ],
    "limit": [
      "NPC policies revalidate conditions and evidence; an approach does not guarantee a viable route.",
      "Políticas de NPC revalidam condições e evidência; aproximação não garante caminho viável."
    ],
    "commands": [
      "set_ability_catalog",
      "activate_ability",
      "cancel_ability",
      "inspect_abilities",
      "inspect_resources"
    ],
    "roots": [
      "combat",
      "npcs"
    ],
    "id": "system-14",
    "sources": [
      "docs/commands.md"
    ],
    "features": [
      [
        "Shared typed control through MCP",
        "Inspectable execution and explicit failure",
        "Versioned authorship or bounded session state"
      ],
      [
        "Controle tipado compartilhado pelo MCP",
        "Execução inspecionável e falha explícita",
        "Autoria versionada ou estado de sessão limitado"
      ]
    ]
  },
  {
    "group": "actions",
    "title": [
      "Life, damage, healing and consequences",
      "Vida, dano, cura e consequências"
    ],
    "body": [
      "Damage/healing and death are transactional, cancelling casts, reservations and motors. Explicit resurrection preserves identity, resources and cooldowns, advances life index and requires fresh evidence for observers unaware of the return.",
      "Dano/cura e morte são transacionais, com cancelamento de casts, reservas e motores. Ressurreição explícita preserva identidade, recursos e cooldowns, avança o índice de vida e exige nova evidência para quem não observou o retorno."
    ],
    "limit": [
      "Resurrecting the same individual differs from template spawning or slot reuse.",
      "Ressurreição do mesmo indivíduo é distinta de nascer por template ou reutilizar um slot."
    ],
    "commands": [
      "inspect_vitality",
      "inspect_abilities"
    ],
    "roots": [
      "combat",
      "character"
    ],
    "id": "system-15",
    "sources": [
      "docs/resurrection.md"
    ],
    "features": [
      [
        "Shared typed control through MCP",
        "Inspectable execution and explicit failure",
        "Versioned authorship or bounded session state"
      ],
      [
        "Controle tipado compartilhado pelo MCP",
        "Execução inspecionável e falha explícita",
        "Autoria versionada ou estado de sessão limitado"
      ]
    ]
  },
  {
    "group": "actions",
    "title": [
      "Projectiles and spatial intent",
      "Projéteis e intenção espacial"
    ],
    "body": [
      "Straight XYZ projectiles with AABB sweeps and first contact. Aim by direction, point or cursor on an explicit plane; automatic aim uses confirmed, frozen visual evidence.",
      "Projéteis XYZ retilíneos com sweep AABB e primeiro contato. Mira por direção, ponto ou cursor em plano explícito; mira automática usa evidência visual confirmada e congelada."
    ],
    "limit": [
      "Homing, ballistics and target prediction remain separate. Areas and tactics have dedicated contracts described in this atlas.",
      "Homing, balística e previsão do alvo seguem separados. Áreas e tática têm contratos próprios, descritos neste atlas."
    ],
    "commands": [
      "aim_ability",
      "aim_ability_at",
      "activate_ability",
      "inspect_abilities"
    ],
    "roots": [
      "combat"
    ],
    "id": "system-16",
    "sources": [
      "docs/commands.md"
    ],
    "features": [
      [
        "Shared typed control through MCP",
        "Inspectable execution and explicit failure",
        "Versioned authorship or bounded session state"
      ],
      [
        "Controle tipado compartilhado pelo MCP",
        "Execução inspecionável e falha explícita",
        "Autoria versionada ou estado de sessão limitado"
      ]
    ]
  },
  {
    "group": "actions",
    "title": [
      "Effects that persist. Rules that dispel.",
      "Efeitos que persistem. Regras que dissipam."
    ],
    "body": [
      "Damage, healing and resource schedules; refresh or independent executions; category-based dispelling, limits and protection. Channeling, external interruption and attributes/buffs add composition without duplicating the ability executor.",
      "Agendas de dano, cura e recursos; renovação ou execuções independentes; dissipação por categoria, limites e proteção. Canalização, interrupção externa e atributos/buffs acrescentam composição sem duplicar o executor de habilidades."
    ],
    "limit": [
      "Categories, effects, duration and quotas are explicit; effects do not execute arbitrary scripts.",
      "Categorias, efeitos, duração e quotas são explícitos; não há script arbitrário executado como efeito."
    ],
    "commands": [
      "inspect_abilities",
      "inspect_resources",
      "inspect_vitality"
    ],
    "roots": [
      "combat"
    ],
    "id": "system-17",
    "sources": [
      "docs/attributes.md"
    ],
    "features": [
      [
        "Shared typed control through MCP",
        "Inspectable execution and explicit failure",
        "Versioned authorship or bounded session state"
      ],
      [
        "Controle tipado compartilhado pelo MCP",
        "Execução inspecionável e falha explícita",
        "Autoria versionada ou estado de sessão limitado"
      ]
    ]
  },
  {
    "group": "evidence",
    "title": [
      "The contract is the interface",
      "O contrato é a interface"
    ],
    "body": [
      "Shared typed dispatcher, command-derived schemas and structured results. MCP over stdio and native controls reach the same API. Projects run without the agent that created them.",
      "Dispatcher tipado compartilhado, schemas derivados dos comandos e resultados estruturados. MCP via stdio e controles nativos chegam à mesma API. Projetos executam sem o agente que os criou."
    ],
    "limit": [
      "MCP is the control plane; simulation advances internally without an AI call per frame.",
      "MCP é o plano de controle; a simulação avança internamente, sem uma chamada de IA por frame."
    ],
    "commands": [
      "inspect",
      "exit"
    ],
    "roots": [
      "tools"
    ],
    "id": "system-18",
    "sources": [
      "docs/commands.md"
    ],
    "features": [
      [
        "Shared typed control through MCP",
        "Inspectable execution and explicit failure",
        "Versioned authorship or bounded session state"
      ],
      [
        "Controle tipado compartilhado pelo MCP",
        "Execução inspecionável e falha explícita",
        "Autoria versionada ou estado de sessão limitado"
      ]
    ]
  },
  {
    "group": "evidence",
    "title": [
      "Inspect beyond appearances",
      "Inspecionar além da aparência"
    ],
    "body": [
      "Queryable state, sources, reasons, resources, routes and decisions. Vision, hearing, noise and route guides, vitality bars and captures through the same renderer.",
      "Estado, fontes, motivos, recursos, caminhos e decisões consultáveis. Guias de visão, audição, ruído e rotas, barras de vida e capturas pelo mesmo renderizador."
    ],
    "limit": [
      "Developer inspection does not enter NPC knowledge.",
      "A inspeção do desenvolvedor não entra no conhecimento dos NPCs."
    ],
    "commands": [
      "inspect",
      "set_debug_view",
      "capture",
      "events"
    ],
    "roots": [
      "tools",
      "render"
    ],
    "id": "system-19",
    "sources": [
      "docs/commands.md"
    ],
    "features": [
      [
        "Shared typed control through MCP",
        "Inspectable execution and explicit failure",
        "Versioned authorship or bounded session state"
      ],
      [
        "Controle tipado compartilhado pelo MCP",
        "Execução inspecionável e falha explícita",
        "Autoria versionada ou estado de sessão limitado"
      ]
    ],
    "media": "editor"
  },
  {
    "group": "evidence",
    "title": [
      "Reproduce. Verify. Refine.",
      "Reproduzir. Verificar. Corrigir."
    ],
    "body": [
      "Tick-inspectable replay and isolated captures without changing the live session. Scenarios with typed checks, finite budgets and reports distinguishing behavior failures from transport errors.",
      "Replay inspecionável por tick e captura isolada sem alterar a partida viva. Cenários com checks tipados, orçamento finito e relatório que distingue falha de comportamento de erro de transporte."
    ],
    "limit": [
      "Replay requires the same build; cross-platform determinism is not a current guarantee.",
      "Replay exige o mesmo build; determinismo entre plataformas não é uma garantia atual."
    ],
    "commands": [
      "save_replay",
      "run_replay",
      "inspect_replay",
      "capture_replay",
      "save_scenario",
      "inspect_scenario",
      "run_scenario"
    ],
    "roots": [
      "tools"
    ],
    "id": "system-20",
    "sources": [
      "docs/commands.md"
    ],
    "features": [
      [
        "Shared typed control through MCP",
        "Inspectable execution and explicit failure",
        "Versioned authorship or bounded session state"
      ],
      [
        "Controle tipado compartilhado pelo MCP",
        "Execução inspecionável e falha explícita",
        "Autoria versionada ou estado de sessão limitado"
      ]
    ]
  },
  {
    "id": "system-21",
    "roots": [
      "combat"
    ],
    "title": [
      "Attributes, buffs & composed damage",
      "Atributos, buffs e dano composto"
    ],
    "body": [
      "Scaled integer resources and configurable numeric/boolean attributes support temporary effects and modifiers. Damage is composed of classified components, with separate source and recipient factors.",
      "Recursos inteiros com escala e atributos numéricos/booleanos configuráveis sustentam efeitos temporários e modificadores. Dano é composto por parcelas classificadas, com fatores do autor e destinatário."
    ],
    "features": [
      [
        "Per-component tags, no fixed element enum",
        "Per-transaction contribution snapshots",
        "Atomic charging/application with full rollback"
      ],
      [
        "Tags por componente, sem lista fixa de elementos",
        "Snapshot de contribuições por transação",
        "Cobrança e aplicação com rollback integral"
      ]
    ],
    "limit": [
      "Storage limits do not imply sustained worst-case 60 Hz.",
      "Limites de armazenamento não equivalem a pior caso sustentado a 60 Hz."
    ],
    "commands": [
      "inspect_attribute",
      "inspect_resources",
      "inspect_abilities"
    ],
    "sources": [
      "docs/attributes.md",
      "docs/damage-components.md",
      "docs/outgoing-damage.md"
    ],
    "status": "implemented"
  },
  {
    "id": "system-22",
    "roots": [
      "combat"
    ],
    "title": [
      "Channel, interrupt & dispel",
      "Canalizar, interromper e dissipar"
    ],
    "body": [
      "An ability can prepare, channel pulses, be cancelled or receive external interruption. Initial or per-pulse charging, cooldown and interruption protection have distinct semantics.",
      "Uma habilidade pode preparar, canalizar pulsos, ser cancelada ou receber interrupção externa. Pagamento inicial ou por pulso, cooldown e proteção de interrupção têm semântica própria."
    ],
    "features": [
      [
        "First pulse after its interval",
        "Cost/vision/range revalidation",
        "Dispelling and cancellation are distinct operations"
      ],
      [
        "Primeiro pulso após intervalo",
        "Revalidação de custo/visão/alcance",
        "Dissipação e cancelamento são operações diferentes"
      ]
    ],
    "limit": [
      "Protection does not implicitly block death, self-cancellation or movement policy.",
      "Proteção não bloqueia implicitamente morte, cancelamento próprio ou política de movimento."
    ],
    "commands": [
      "activate_ability",
      "cancel_ability",
      "inspect_abilities"
    ],
    "sources": [
      "docs/channeling.md",
      "docs/interrupts.md",
      "docs/dispel.md"
    ],
    "status": "implemented"
  },
  {
    "id": "system-23",
    "roots": [
      "combat",
      "npcs"
    ],
    "title": [
      "Areas, auras & intelligent support",
      "Áreas, auras e apoio inteligente"
    ],
    "body": [
      "Instant spheres, persistent volumes and pulsing auras reuse relationships, occlusion and per-recipient transactions. NPCs choose injured or dispellable allies from authorized evidence.",
      "Esferas instantâneas, volumes persistentes e auras pulsantes reutilizam relações, oclusão e transações por destinatário. NPCs escolhem aliados feridos ou dissipáveis a partir de evidência autorizada."
    ],
    "features": [
      [
        "XYZ sampling per pulse",
        "Fixed or source-anchored effects",
        "Triage and support reservations by identity/life"
      ],
      [
        "Amostragem XYZ por pulso",
        "Âncora fixa ou no emissor",
        "Triagem e reservas de apoio por identidade/vida"
      ]
    ],
    "limit": [
      "Pulsing auras do not imply continuous enter/exit callbacks or universal tactical immunity.",
      "Aura pulsante não implica callback contínuo enter/exit ou imunidade tática universal."
    ],
    "commands": [
      "inspect_abilities",
      "inspect_npc_cooperation"
    ],
    "sources": [
      "docs/instant-area.md",
      "docs/persistent-area.md",
      "docs/auras.md",
      "docs/ally-triage.md",
      "docs/dispel-triage.md"
    ],
    "status": "implemented"
  },
  {
    "id": "system-24",
    "roots": [
      "modeling",
      "tools"
    ],
    "title": [
      "Editor, workspaces & projects",
      "Editor, áreas e projetos"
    ],
    "body": [
      "Native editor with configurable areas, menus, project library and integrated file browsing. Creating, opening, favoriting and discovering projects shares documented operations with agents.",
      "Editor nativo com áreas configuráveis, menus, biblioteca de projetos e navegador integrado. Criar, abrir, favoritar e descobrir projetos compartilha operações documentadas com agentes."
    ],
    "features": [
      [
        "Persistent layouts and workspaces",
        "Scenes and game entry point",
        "Explicit preview, confirm and cancel"
      ],
      [
        "Layouts persistidos e workspaces",
        "Cenas e ponto de entrada do jogo",
        "Preview, confirmação e cancelamento explícitos"
      ]
    ],
    "limit": [
      "The UI capture is historical; it does not promise identical current menus.",
      "Captura de UI é histórica; não promete identidade visual de todos os menus atuais."
    ],
    "commands": [
      "inspect_editor",
      "edit_editor",
      "create_project",
      "open_project",
      "inspect_project_library",
      "capture_editor"
    ],
    "sources": [
      "docs/editor-ui.md",
      "docs/projects.md",
      "docs/editor-workflows.md"
    ],
    "status": "implemented",
    "media": "editor"
  },
  {
    "id": "system-25",
    "roots": [
      "modeling"
    ],
    "title": [
      "Component modeling",
      "Modelagem por componentes"
    ],
    "body": [
      "Vertices, edges, faces and islands are selectable through session/revision references. Extrusion, inset and bevel produce real geometry through a shared editor/MCP kernel.",
      "Vértices, arestas, faces e ilhas são selecionáveis por referências de sessão/revisão. Extrusão, inset e bevel produzem geometria real por um kernel compartilhado entre editor e MCP."
    ],
    "features": [
      [
        "Connected extrusion preserves holes",
        "Convex planar inset",
        "Planar bevel along convex-solid chains"
      ],
      [
        "Extrusão conectada preserva furos",
        "Inset plano convexo",
        "Bevel plano em cadeias de sólido convexo"
      ]
    ],
    "limit": [
      "Curved bevels, concave insets and global self-intersection detection are not current capabilities.",
      "Bevel curvo, inset côncavo e detecção global de auto-interseção não são capacidades atuais."
    ],
    "commands": [
      "model_editor_mesh",
      "select_mesh_components",
      "inspect_mesh_topology",
      "preview_mesh_edit"
    ],
    "sources": [
      "docs/editor-modeling.md",
      "docs/mesh-topology.md"
    ],
    "status": "implemented",
    "media": "modeling"
  },
  {
    "id": "system-26",
    "roots": [
      "modeling"
    ],
    "title": [
      "Sculpting with local detail",
      "Escultura com detalhe local"
    ],
    "body": [
      "Draw, move, smooth, flatten, pinch, crease, scrape, fill and relax are geometric operations. Brushes can refine the affected region, with strength/radius controls and one undo per gesture.",
      "Volume, mover, suavizar, achatar, pinçar, vinco, raspar, preencher e relaxar são operações geométricas. O pincel pode refinar a região tocada, com força/raio e um único undo por gesto."
    ],
    "features": [
      [
        "Preview and commit from one base",
        "Under-brush refinement",
        "Guards against inversion and data loss"
      ],
      [
        "Preview e commit sobre a mesma base",
        "Detalhar sob o pincel",
        "Guardas contra inversão e perda de dados"
      ]
    ],
    "limit": [
      "Masks, tablet support, universal remeshing and full DCC workflows remain separate.",
      "Máscaras, tablet, remesh universal e todos os fluxos de um DCC completo permanecem separados."
    ],
    "commands": [
      "edit_mesh",
      "preview_mesh_edit",
      "inspect_mesh_selection"
    ],
    "sources": [
      "docs/sculpt-brushes.md",
      "docs/editor-sculpt.md"
    ],
    "status": "implemented",
    "media": "sculpt"
  },
  {
    "id": "system-27",
    "roots": [
      "modeling",
      "botany"
    ],
    "title": [
      "Curves, sections & sweeps",
      "Curvas, seções e sweep"
    ],
    "body": [
      "Editable profiles and trajectories compose branches, structures and swept forms. Sampling, radius, taper and oriented frames remain authored data, with resolution and domain inspection.",
      "Perfis editáveis e trajetórias compõem galhos, estruturas e formas extrudidas. Amostragem, raio, afilamento e frames orientados permanecem dados autorais, com inspeção da resolução e do domínio."
    ],
    "features": [
      [
        "Planning before publication",
        "Editable sections and lofts",
        "Boundary and sampling diagnostics"
      ],
      [
        "Planejamento antes de publicar",
        "Seções e lofts editáveis",
        "Diagnóstico de bordas e amostragem"
      ]
    ],
    "limit": [
      "Sweep families have validated domains; they are not an unrestricted CAD kernel.",
      "As famílias de sweep têm domínios validados; não são um kernel CAD irrestrito."
    ],
    "commands": [
      "plan_mesh_sweep",
      "preview_mesh_ring_loft",
      "edit_plant",
      "inspect_plant_sampling"
    ],
    "sources": [
      "docs/mesh-sweep.md",
      "docs/plant-trajectories.md",
      "docs/curved-plants.md"
    ],
    "status": "implemented",
    "media": "roots"
  },
  {
    "id": "system-28",
    "roots": [
      "modeling",
      "botany"
    ],
    "title": [
      "Organic junctions & regeneration",
      "Junções orgânicas e regeneração"
    ],
    "body": [
      "Collars, branches and connected surfaces use native junctions with closure validation. Canonical sources and material addresses allow regeneration with detail preservation when valid correspondence exists.",
      "Colares, ramos e superfícies conectadas usam junções nativas com validação de fechamento. Fontes canônicas e endereços materiais permitem regenerar preservando detalhes quando há correspondência válida."
    ],
    "features": [
      [
        "Distinct forks and lateral branches",
        "Transactional preview/publication",
        "Explicit rejection of sculpt loss"
      ],
      [
        "Forquilhas e ramos laterais distintos",
        "Preview e publicação transacional",
        "Recusa explícita de perda de escultura"
      ]
    ],
    "limit": [
      "There is no general boolean union or universal C1-continuity guarantee.",
      "Não há união booleana geral nem continuidade C1 universal garantida."
    ],
    "commands": [
      "preview_mesh_junction_union",
      "edit_plant",
      "botany"
    ],
    "sources": [
      "docs/organic-continuity.md",
      "docs/botany-collar.md",
      "docs/mesh-preservation.md"
    ],
    "status": "implemented",
    "media": "roots"
  },
  {
    "id": "system-29",
    "roots": [
      "render",
      "modeling"
    ],
    "title": [
      "Procedural materials & surface details",
      "Materiais procedurais e detalhes"
    ],
    "body": [
      "Rest-space 3D fields describe color, bump, roughness and surface detail. Veins, grooves and triplanar patterns follow form without sliding through world space during animation.",
      "Campos 3D em repouso descrevem cor, bump, rugosidade e detalhes de superfície. Nervuras, sulcos e padrões triplanares acompanham a forma sem deslizar pelo mundo durante a animação."
    ],
    "features": [
      [
        "Seeded noise, ridges and cells",
        "Detail domains and layers",
        "Material coordinates preserved through skinning"
      ],
      [
        "Ruído, cristas e células seeded",
        "Domínios e camadas de detalhes",
        "Coordenadas materiais preservadas no skinning"
      ]
    ],
    "limit": [
      "Bump does not alter physical geometry; a pattern is not constitutive material simulation.",
      "Bump não altera a geometria física; um padrão não é simulação constitutiva do material."
    ],
    "commands": [
      "edit_mesh",
      "inspect_mesh_material_domain",
      "preview_mesh_material_finish"
    ],
    "sources": [
      "docs/procedural-materials.md",
      "docs/surface-details.md",
      "docs/layered-patterns.md",
      "docs/triplanar-details.md"
    ],
    "status": "implemented",
    "media": "roots"
  },
  {
    "id": "system-30",
    "roots": [
      "render"
    ],
    "title": [
      "Dielectrics, metals & environment",
      "Dielétricos, metais e ambiente"
    ],
    "body": [
      "GGX surface response, roughness, Fresnel and metallicity combine with direct light and prefiltered procedural environment. One material system serves wood, stone, leaves or metal.",
      "Resposta de superfície GGX, rugosidade, Fresnel e metalicidade se combinam com luz direta e ambiente procedural pré-filtrado. O mesmo material pode servir madeira, pedra, folhas ou metal."
    ],
    "features": [
      [
        "View-dependent specular light",
        "Reusable cubemap/LUT environment",
        "Surface normals evaluated in the pipeline"
      ],
      [
        "Iluminação especular dependente da vista",
        "Ambiente cubemap/LUT reutilizado",
        "Normais de superfície avaliadas no pipeline"
      ]
    ],
    "limit": [
      "This is not ray tracing, dynamic GI, HDR import or complete local reflections.",
      "Não é ray tracing, GI dinâmica, importador HDR ou reflexão local completa."
    ],
    "commands": [
      "set_light",
      "edit_mesh",
      "capture"
    ],
    "sources": [
      "docs/dielectric-response.md",
      "docs/metallic-response.md",
      "docs/procedural-environment.md"
    ],
    "status": "implemented",
    "media": "horizon"
  },
  {
    "id": "system-31",
    "roots": [
      "render"
    ],
    "title": [
      "Shadows, cameras & outlines",
      "Sombras, câmera e contornos"
    ],
    "body": [
      "Directional shadows and environment have separate costs and sources. Picking and outlines respect visible coverage; guides and selection are observation tools, not NPC knowledge.",
      "Sombras direcionais e ambiente têm custos e fontes distintos. Picking e contornos respeitam cobertura visível; guias e seleção são ferramentas de observação, não conhecimento dos NPCs."
    ],
    "features": [
      [
        "Controllable cameras and lighting",
        "Object or component selection",
        "Guides after atmospheric finishing"
      ],
      [
        "Câmeras e iluminação controláveis",
        "Seleção por objeto ou componente",
        "Guias depois do acabamento atmosférico"
      ]
    ],
    "limit": [
      "Bounded directional shadows do not equal global illumination or complete physical transparency.",
      "Sombra direcional limitada não equivale a iluminação global ou transparência física completa."
    ],
    "commands": [
      "set_camera",
      "set_light",
      "select_at",
      "set_debug_view"
    ],
    "sources": [
      "docs/directional-shadows.md",
      "docs/atmosphere-silhouettes.md"
    ],
    "status": "implemented",
    "media": "editor"
  },
  {
    "id": "system-32",
    "roots": [
      "render"
    ],
    "title": [
      "Content-based shared geometry",
      "Geometria compartilhada por conteúdo"
    ],
    "body": [
      "Repeated static assets share GPU geometry while transform, tint and material vary per instance. Authoring capacity no longer depends on the old 16-asset or 32-instance caps.",
      "Assets estáticos repetidos compartilham geometria GPU, enquanto transformação, tinta e material variam por instância. A capacidade autoral não depende dos antigos tetos de 16 assets ou 32 instâncias."
    ],
    "features": [
      [
        "80 bytes per instance in this layout",
        "Content-driven u16/u32 indices",
        "Editable source and collision preserved"
      ],
      [
        "80 bytes por instância neste layout",
        "Índices u16/u32 escolhidos pelo conteúdo",
        "Fonte editável e colisão preservadas"
      ]
    ],
    "limit": [
      "Sharing geometry does not eliminate the cost of drawing millions of triangles across copies.",
      "Compartilhar geometria não elimina o custo de desenhar milhões de triângulos por cópia."
    ],
    "commands": [
      "inspect_meshes",
      "inspect_viewport_performance"
    ],
    "sources": [
      "docs/mesh-resources.md",
      "docs/small-geometry-cost.md"
    ],
    "status": "implemented",
    "media": "horizon"
  },
  {
    "id": "system-33",
    "roots": [
      "render"
    ],
    "title": [
      "Conservative visibility",
      "Visibilidade conservadora"
    ],
    "body": [
      "Camera and light reject candidates outside coverage before preparation and submission. Bounds derive from source and pose, including components and articulated presentation.",
      "Câmera e luz descartam candidatos fora da cobertura antes de preparar e enviar trabalho. Bounds são derivados da fonte e da pose, incluindo componentes e apresentação articulada."
    ],
    "features": [
      [
        "Camera and shadow frusta",
        "Fallback when proof is unavailable",
        "Preserved order under overlap"
      ],
      [
        "Frustum de câmera e sombra",
        "Fallback para casos sem prova",
        "Ordem preservada onde há sobreposição"
      ]
    ],
    "limit": [
      "Frustum culling differs from occlusion and does not simplify surviving geometry.",
      "Frustum culling é distinto de oclusão e não simplifica a geometria sobrevivente."
    ],
    "commands": [
      "inspect_viewport_performance",
      "capture"
    ],
    "sources": [
      "docs/render-visibility.md"
    ],
    "status": "implemented",
    "media": "horizon"
  },
  {
    "id": "system-34",
    "roots": [
      "render"
    ],
    "title": [
      "Two-scale GPU occlusion",
      "Oclusão GPU em duas escalas"
    ],
    "body": [
      "Current depth, reverse-Z, conservative reduction and stable compaction avoid drawing genuinely occluded instances. Small candidates can use 4×4 blocks; others retain 16×16 or fallback.",
      "Profundidade corrente, reverse-Z, redução conservadora e compactação estável evitam desenhar instâncias realmente ocultas. Candidatos pequenos podem usar blocos 4×4; os demais conservam 16×16 ou fallback."
    ],
    "features": [
      [
        "No per-frame decision readback",
        "Partial coverage never invents an occluder",
        "1.7–7.9% lower draw+wait in six measured cases"
      ],
      [
        "Decisão sem readback por frame",
        "Cobertura parcial não inventa oclusor",
        "1,7–7,9% menos draw+espera nos seis casos medidos"
      ]
    ],
    "limit": [
      "Specified M4 Pro/Metal scenes and resolutions; draw+wait is not a GPU timestamp or gameplay FPS.",
      "M4 Pro/Metal, resoluções e cenas especificadas; draw+espera não é timestamp GPU nem FPS de gameplay."
    ],
    "commands": [
      "inspect_viewport_performance"
    ],
    "sources": [
      "docs/render-occlusion.md",
      "docs/fine-occlusion.md",
      "docs/adaptive-occluders.md"
    ],
    "status": "implemented",
    "media": "horizon"
  },
  {
    "id": "system-35",
    "roots": [
      "render"
    ],
    "title": [
      "Residency & consumed attributes",
      "Residência e atributos consumidos"
    ],
    "body": [
      "Eligible static geometry remains resident, including large unique meshes. Shaders without procedural channels use minimal position/normal streams; absent fields are not prepared unnecessarily.",
      "Geometria estática elegível fica residente, inclusive malhas únicas grandes. Shaders sem canais procedurais usam streams mínimos de posição e normal; campos ausentes não são preparados inutilmente."
    ],
    "features": [
      [
        "24/48/64 bytes according to consumed channels",
        "Source/representation-keyed caches",
        "Edits invalidate layout and representation"
      ],
      [
        "24/48/64 bytes conforme canais usados",
        "Cache por fonte e representação",
        "Edição invalida layout e representação"
      ]
    ],
    "limit": [
      "CPU source remains resident. Buffer savings are not total game RAM measurements.",
      "A fonte CPU permanece residente. Economia de buffer não é uma medida da RAM total do jogo."
    ],
    "commands": [
      "inspect_viewport_performance",
      "inspect_meshes"
    ],
    "sources": [
      "docs/minimal-vertex-stream.md",
      "docs/static-residency.md"
    ],
    "status": "implemented",
    "media": "sunset"
  },
  {
    "id": "system-36",
    "roots": [
      "render",
      "world"
    ],
    "title": [
      "Distant representations & atmosphere",
      "Representações distantes e atmosfera"
    ],
    "body": [
      "Compiled packages preserve source and admit cheaper representations when contribution bounds allow. Atmosphere separates sky and mountains; optional edge finishing has its own cost.",
      "Pacotes compilados preservam a fonte e admitem representações mais baratas quando os limites de contribuição permitem. Atmosfera separa céu e montanhas; acabamento de borda é opcional e tem custo próprio."
    ],
    "features": [
      [
        "ATM derivation validation",
        "Edits invalidate packages",
        "Shadows/rigs retain source fallback"
      ],
      [
        "ATM com validação de derivação",
        "Edição invalida o pacote",
        "Sombras/rigs conservam fallback de fonte"
      ]
    ],
    "limit": [
      "The mountain comparison reduced buffers, not curved triangles; timing varied by scene.",
      "O comparativo de montanhas reduziu buffers, não triângulos curvos; tempo variou por cenário."
    ],
    "commands": [
      "edit_scene",
      "set_light",
      "inspect_viewport_performance"
    ],
    "sources": [
      "docs/atmosphere.md",
      "docs/atmosphere-silhouettes.md",
      "docs/render-optimization-comparison.md"
    ],
    "status": "implemented",
    "media": "sunset"
  },
  {
    "id": "system-37",
    "roots": [
      "render"
    ],
    "title": [
      "Virtualized geometry & LOD (research)",
      "Geometria virtualizada e LOD (pesquisa)"
    ],
    "body": [
      "The Ignis Nanites research track includes candidate compilation while preserving boundaries and components; a verifier recomputes surface/normal correspondence bounds. Runtime has error criteria and source fallback.",
      "A frente de pesquisa Nanites da Ignis inclui compilação de candidatos mantendo fronteiras e componentes; um verificador recompõe limites de correspondência de superfície e normal. Runtime tem critérios de erro e fallback da fonte."
    ],
    "features": [
      [
        "IGL1 sidecar separate from authorship",
        "Source and physics unchanged",
        "Control plane: 2,048 → 512 triangles with identical PNG"
      ],
      [
        "Sidecar IGL1 separado da autoria",
        "Fonte e física intactas",
        "Plano de controle 2.048 → 512 triângulos com PNG idêntico"
      ]
    ],
    "limit": [
      "Experimental and disabled by default. The forest produced no useful package; this is not complete Nanite or demonstrated universal superiority.",
      "Experimental e desligado por padrão. A floresta não teve pacote útil; não é Nanite completo nem superioridade universal demonstrada."
    ],
    "commands": [
      "compile_mesh_lods",
      "load_mesh_lods",
      "set_mesh_lod_settings",
      "inspect_mesh_lods"
    ],
    "sources": [
      "docs/automatic-lod.md",
      "docs/regional-lod.md"
    ],
    "status": "experimental"
  },
  {
    "id": "system-38",
    "roots": [
      "modeling",
      "render"
    ],
    "title": [
      "Composite objects & hierarchies",
      "Objetos compostos e hierarquias"
    ],
    "body": [
      "Explicit bindings organize pieces without destroying instancing. Groups can be rejected by conservative bounds; partially visible or deformed groups retain per-piece tests.",
      "Vínculos explícitos organizam peças sem destruir instancing. Conjuntos podem ser descartados por bounds conservadores; grupos parcialmente visíveis ou deformados mantêm testes por peça."
    ],
    "features": [
      [
        "Objects independent from actors/collision",
        "Instance- or bone-based attachments",
        "Move, copy and transform islands"
      ],
      [
        "Objetos independentes de atores/colisão",
        "Attachments por instância ou osso",
        "Mover, copiar e transformar ilhas"
      ]
    ],
    "limit": [
      "Grouping is not collective appearance LOD or a boolean union of pieces.",
      "Agrupamento não é LOD coletivo da aparência nem fusão booleana das peças."
    ],
    "commands": [
      "inspect_mesh_objects",
      "inspect_mesh_islands",
      "edit_mesh"
    ],
    "sources": [
      "docs/mesh-objects.md",
      "docs/mesh-attachments.md",
      "docs/mesh-island-transform.md"
    ],
    "status": "implemented",
    "media": "modeling"
  },
  {
    "id": "system-39",
    "roots": [
      "botany"
    ],
    "title": [
      "Procedural plant generation",
      "Geração procedural de plantas"
    ],
    "body": [
      "A plant starts as a graph of organs, oriented curves, sections and contours—not an opaque mesh. The Rust compiler publishes native geometry and retains the editable recipe in the scene.",
      "Uma planta nasce de um grafo de órgãos, curvas orientadas, seções e contornos — não de uma malha opaca. O compilador Rust publica geometria nativa e conserva a receita editável na cena."
    ],
    "features": [
      [
        "Palm, plane tree, eucalyptus and fig trees",
        "Vegetation editor and MCP share operations",
        "Persistent organs and material addresses"
      ],
      [
        "Palmeira, plátano, eucalipto e figueiras",
        "Editor Vegetação e MCP compartilham operações",
        "Órgãos e endereços materiais persistentes"
      ]
    ],
    "limit": [
      "Profiles are authored, not universal botanical calibration or automatic photo reconstruction.",
      "Perfis são autorais, não calibração botânica universal ou reconstrução automática de fotografias."
    ],
    "commands": [
      "botany"
    ],
    "sources": [
      "docs/botany-integration.md",
      "docs/botany-reference-refinement.md"
    ],
    "status": "implemented",
    "media": "botany"
  },
  {
    "id": "system-40",
    "roots": [
      "botany"
    ],
    "title": [
      "Age, vigor & development",
      "Idade, vigor e desenvolvimento"
    ],
    "body": [
      "Age in years coordinates height, canopy, thickening, organ birth and current foliage. Vigor modifies development; palms do not automatically inherit the woody growth law.",
      "Idade em anos coordena altura, copa, engrossamento, nascimento de órgãos e folhagem corrente. Vigor modifica o desenvolvimento; palmeiras não recebem automaticamente a lei lenhosa."
    ],
    "features": [
      [
        "Independent growth curves",
        "Young/middle/adult snapshots",
        "Preserved site IDs and parentage"
      ],
      [
        "Curvas de crescimento independentes",
        "Snapshots young/middle/adult",
        "IDs de sítio e parentesco conservados"
      ]
    ],
    "limit": [
      "Age is profile configuration, not real dating or decades of temporal simulation.",
      "Idade é configuração por perfil, não datação real nem simulação temporal de décadas."
    ],
    "commands": [
      "botany"
    ],
    "sources": [
      "docs/botany-age.md"
    ],
    "status": "implemented",
    "media": "mature"
  },
  {
    "id": "system-41",
    "roots": [
      "botany"
    ],
    "title": [
      "Volumetric canopies & branching",
      "Copas volumétricas e ramificação"
    ],
    "body": [
      "Subcanopy partitions, direction, load-based radii, interior buds and additional orders generate branching architecture. Terminal forks join three collars; lateral grafts and sheaths have distinct roles.",
      "Partições de subcopa, direção, raios por carga, brotos interiores e ordens adicionais geram arquitetura ramificada. Forquilhas terminais unem três colares; ramos laterais e bainhas têm papéis distintos."
    ],
    "features": [
      [
        "Reproducible seed",
        "Pruning rebuilds defined continuity",
        "Leaves are real geometry, not photographs"
      ],
      [
        "Seed reproduzível",
        "Poda recompõe continuidade prevista",
        "Folhas são geometria real, não fotografias"
      ]
    ],
    "limit": [
      "Species/photo resemblance requires separate artistic acceptance; generation does not identify species.",
      "Parecer com uma espécie ou foto é aceite artístico próprio; a geração não identifica espécies."
    ],
    "commands": [
      "botany"
    ],
    "sources": [
      "docs/botany-architecture.md",
      "docs/botany-branching.md",
      "docs/botany-rs.md"
    ],
    "status": "implemented",
    "media": "botany"
  },
  {
    "id": "system-42",
    "roots": [
      "botany"
    ],
    "title": [
      "Roots, buttresses & root collars",
      "Raízes, contrafortes e colo"
    ],
    "body": [
      "Radial, taproot and fibrous systems produce editable axes. Buttresses add walls, ridges, meanders and ribs; the collar connects roots and trunk through closed native skin.",
      "Sistemas radiais, pivotantes e fibrosos produzem eixos editáveis. Contrafortes acrescentam parede, crista, meandro e nervuras; o colo conecta raízes e tronco por uma pele nativa fechada."
    ],
    "features": [
      [
        "Seeded regeneration",
        "Stitched skin without intersecting fallback",
        "Explicit sculpt-preservation policy"
      ],
      [
        "Regeneração seeded",
        "Pele costurada sem fallback atravessado",
        "Política explícita de preservar escultura"
      ]
    ],
    "limit": [
      "Soil interaction, anastomosis and universal tangential continuity are not implemented.",
      "Contato com solo, anastomose e continuidade tangencial universal ainda não estão implementados."
    ],
    "commands": [
      "botany"
    ],
    "sources": [
      "docs/botany-roots.md",
      "docs/botany-buttress.md",
      "docs/botany-collar.md"
    ],
    "status": "implemented",
    "media": "roots"
  },
  {
    "id": "system-43",
    "roots": [
      "botany"
    ],
    "title": [
      "Botanical dynamics laboratory",
      "Dinâmica botânica em laboratório"
    ],
    "body": [
      "An XPBD laboratory keyed by identity, mass and momentum supports deformation, growth/pruning and contact proxies. Rest curves are the same source used by native authoring.",
      "Um ensaio XPBD por identidade, massa e momento permite deformação, crescimento/poda e proxies de contato. A curva de repouso é a mesma fonte usada pela autoria nativa."
    ],
    "features": [
      [
        "Explicit anchors",
        "Wind and sway experiments",
        "Active static-collider policy"
      ],
      [
        "Âncoras explícitas",
        "Ensaios com vento e balanço",
        "Política para colisor estático ativo"
      ]
    ],
    "limit": [
      "Session laboratory separate from Rapier gameplay, saves and networking; not universal constitutive tissue or soil.",
      "Ensaio de sessão separado do gameplay Rapier, saves e rede; não é tecido ou solo constitutivo universal."
    ],
    "commands": [
      "botany"
    ],
    "sources": [
      "docs/botany-integration.md",
      "docs/botany-collar.md"
    ],
    "status": "implemented",
    "media": "roots"
  },
  {
    "id": "system-44",
    "roots": [
      "botany",
      "world"
    ],
    "title": [
      "Groundcover & tall grass",
      "Cobertura de chão e mato alto"
    ],
    "body": [
      "Authored libraries combine blades, stems, spikes, panicles, plumes and flowers. Real source/distribution variations replace merely scaling one tuft.",
      "Bibliotecas autorais combinam folhas, hastes, espigas, panículas, plumas e flores. Variações reais de fonte e distribuição substituem a simples ampliação do mesmo tufo."
    ],
    "features": [
      [
        "12 low-cover types, 36 clumps",
        "8 tall types, 24 variants",
        "Correlated scale/rotation/tint instancing"
      ],
      [
        "12 tipos baixos, 36 tufos",
        "8 tipos altos, 24 variações",
        "Instâncias com escala/rotação/tinta correlacionadas"
      ]
    ],
    "limit": [
      "Names describe silhouettes, not calibrated species. Instancing does not guarantee zero per-copy cost.",
      "Nomes descrevem silhueta, não espécie calibrada. Instancing não garante custo zero por cópia."
    ],
    "commands": [
      "botany",
      "edit_mesh"
    ],
    "sources": [
      "docs/groundcover-assets.md",
      "docs/tallgrass-assets.md"
    ],
    "status": "implemented",
    "media": "groundcover"
  },
  {
    "id": "system-45",
    "roots": [
      "character",
      "modeling"
    ],
    "title": [
      "Parametric human body",
      "Corpo humano paramétrico"
    ],
    "body": [
      "A native source combines proportions, body mass, musculature, posture and local adjustments. Regeneration preserves recipe/bake and rig under explicit preservation policies.",
      "Uma fonte nativa combina proporções, massa corporal, musculatura, postura e ajustes locais. Regeneração conserva recipe/bake e rig com políticas de preservação explícitas."
    ],
    "features": [
      [
        "262 channels in the current source",
        "469 targets and 163 bones in the refined base",
        "Editable masculine and feminine forms"
      ],
      [
        "262 canais na fonte atual",
        "469 alvos e 163 ossos na base refinada",
        "Formas masculinas e femininas editáveis"
      ]
    ],
    "limit": [
      "Graphic base derived from MakeHuman CC0 data; builds do not certify anatomy or human art.",
      "Base gráfica derivada de dados CC0 MakeHuman; anatomia e arte humana não são certificadas por builds."
    ],
    "commands": [
      "inspect_character_source",
      "connect_character_source",
      "set_character_parameters",
      "regenerate_character"
    ],
    "sources": [
      "docs/character-base.md",
      "docs/character-nose.md"
    ],
    "status": "implemented",
    "media": "character"
  },
  {
    "id": "system-46",
    "roots": [
      "character"
    ],
    "title": [
      "Face, ears & 3D handles",
      "Rosto, orelhas e manipuladores 3D"
    ],
    "body": [
      "Local XYZ fields adjust anatomical regions with optional symmetry while preserving defined neighborhoods. Lips, Cupid’s bow, nose and ears have dedicated controls and bounds.",
      "Campos locais XYZ ajustam regiões anatômicas com simetria opcional, preservando vizinhanças previstas. Lábios, arco do cupido, nariz e orelhas possuem controles e limites próprios."
    ],
    "features": [
      [
        "67 XYZ points in source6",
        "12 ear shapes",
        "Preview/commit and saved configuration"
      ],
      [
        "67 pontos XYZ na fonte6",
        "12 formas de orelha",
        "Preview/commit e configuração persistida"
      ]
    ],
    "limit": [
      "This is not facial capture, universal expressions, complete MetaHuman or entirely original Ignis anatomy.",
      "Não é captura facial, expressão universal, MetaHuman completo ou anatomia original integral da Ignis."
    ],
    "commands": [
      "preview_character_handle",
      "set_character_handle",
      "set_character_handle_view",
      "save_character_configuration"
    ],
    "sources": [
      "docs/character-controls.md",
      "docs/character-cupid.md",
      "docs/character-nose.md",
      "docs/character-smoothing.md"
    ],
    "status": "implemented",
    "media": "character"
  },
  {
    "id": "system-47",
    "roots": [
      "character",
      "npcs"
    ],
    "title": [
      "Rigs, clips, retargeting & IK",
      "Rig, clipes, retarget e IK"
    ],
    "body": [
      "Native hierarchies, weights and clips drive individual presentation. Authored retargeting maps rest poses and scale; visual IK queries physical contact without moving the collider to correct a pose.",
      "Hierarquias, pesos e clipes nativos alimentam a apresentação de indivíduos. Retarget em autoria mapeia repousos e escala; IK visual consulta contato físico, sem mover o collider para corrigir a pose."
    ],
    "features": [
      [
        "Source-independent baked clips",
        "Up to 8 configurable IK chains",
        "Planar root motion delivered to KCC"
      ],
      [
        "Bake de clipe independente da fonte",
        "Até 8 cadeias IK configuráveis",
        "Root motion planar entregue ao KCC"
      ]
    ],
    "limit": [
      "Mapping is explicit; no automatic anatomical retarget or universal foot locking/pelvis solve.",
      "Mapeamento é explícito; sem retarget anatômico automático ou foot locking/pelve universal."
    ],
    "commands": [
      "edit_mesh",
      "inspect_mesh_pose",
      "inspect_npc_presentation"
    ],
    "sources": [
      "docs/npc-presentation-complete.md"
    ],
    "status": "implemented",
    "media": "npc"
  },
  {
    "id": "system-48",
    "roots": [
      "npcs"
    ],
    "title": [
      "Explainable intelligence & personality",
      "Inteligência e personalidade explicáveis"
    ],
    "body": [
      "Intelligence controls deliberation effort and cadence without granting wall vision. Anxiety, caution, courage, curiosity, aggression, sociability and other traits vary independently per individual.",
      "Inteligência controla esforço e cadência de deliberação, sem dar visão através de paredes. Ansiedade, cautela, coragem, curiosidade, agressividade, sociabilidade e outros traços variam independentemente por indivíduo."
    ],
    "features": [
      [
        "Seeded identity-based variation",
        "Inspectable scores and reasons",
        "Cross-cutting decision influence"
      ],
      [
        "Variação seeded por identidade",
        "Scores e motivos inspecionáveis",
        "Influência transversal nas decisões"
      ]
    ],
    "limit": [
      "No mandatory LLM; traits are an authored model, not complete human psychology.",
      "LLM não é obrigatório; traços são um modelo autoral, não psicologia humana completa."
    ],
    "commands": [
      "inspect_npc_mind",
      "inspect_objectives"
    ],
    "sources": [
      "docs/npc-mind.md",
      "docs/npc-v1-closure.md"
    ],
    "status": "implemented",
    "media": "npc"
  },
  {
    "id": "system-49",
    "roots": [
      "npcs",
      "combat"
    ],
    "title": [
      "Tactics, orders & cooperation",
      "Tática, ordens e cooperação"
    ],
    "body": [
      "Desired range, cover and flanking produce destinations constrained by navigation and known evidence. Group orders, formations and support reservations preserve ownership, identity and life.",
      "Alcance desejado, cobertura e flanco produzem destinos sujeitos à navegação e à evidência conhecida. Ordens de grupo, formação e reservas de apoio preservam dono, identidade e vida."
    ],
    "features": [
      [
        "Game-configurable roles",
        "Per-target support leases",
        "Trait-influenced preferences"
      ],
      [
        "Papéis configuráveis pelo jogo",
        "Leases de apoio por alvo",
        "Preferências influenciadas por traços"
      ]
    ],
    "limit": [
      "A suggested position does not prove a viable route; there is no omniscient global danger sensor.",
      "Uma posição sugerida não prova rota viável; não há um sensor onisciente global de perigo."
    ],
    "commands": [
      "order_npc",
      "order_npc_group",
      "inspect_npc_tactics",
      "inspect_npc_cooperation"
    ],
    "sources": [
      "docs/npc-tactics-cooperation.md"
    ],
    "status": "implemented",
    "media": "npc"
  },
  {
    "id": "system-50",
    "roots": [
      "npcs"
    ],
    "title": [
      "Needs, objects & dialogue",
      "Necessidades, objetos e diálogo"
    ],
    "body": [
      "Cyclic schedules and resource-based needs drive routines. Interactions apply typed effects to objects or game state; structured dialogues retain revisions and life references.",
      "Agendas cíclicas e necessidades baseadas em recursos conduzem rotinas. Interações aplicam efeitos tipados em objetos ou estado de jogo; diálogos estruturados mantêm revisão e referências de vida."
    ],
    "features": [
      [
        "Rollback-safe interactions",
        "Explicit object collision/vision updates",
        "Inspectable dialogue choices"
      ],
      [
        "Interações com rollback",
        "Objetos atualizam colisão/visão explicitamente",
        "Escolhas de diálogo inspecionáveis"
      ]
    ],
    "limit": [
      "This is not autonomous natural language or unrestricted memory/Scene mutation.",
      "Não é linguagem natural autônoma ou mutação livre de memória/Scene."
    ],
    "commands": [
      "interact",
      "talk_npc",
      "choose_dialogue",
      "inspect_npc_routines",
      "inspect_dialogue"
    ],
    "sources": [
      "docs/npc-routines-interactions.md"
    ],
    "status": "implemented",
    "media": "npc"
  },
  {
    "id": "system-51",
    "roots": [
      "npcs"
    ],
    "title": [
      "Network generation & circulation",
      "Descoberta de rede e circulação"
    ],
    "body": [
      "Networks can be generated from declared geometry/regions, admitted by body and capability. Ground and air circulation query anonymous neighbors; committed actions preserve their movement authority.",
      "A rede pode ser gerada da geometria e região declaradas, com admissão por corpo/capacidade. Circulação terrestre e aérea consulta vizinhos anonimamente; ações comprometidas preservam sua autoridade de movimento."
    ],
    "features": [
      [
        "Ground KCC and jump/fly/glide segments",
        "Observable planning and truncation",
        "Aerial avoidance considers curves/collision"
      ],
      [
        "KCC terrestre e trechos jump/fly/glide",
        "Planejamento e truncamento observáveis",
        "Desvio aéreo considera curvas e colisão"
      ]
    ],
    "limit": [
      "Bounded networks are not a world navmesh. Committed jump/glide segments do not receive arbitrary later avoidance.",
      "Rede limitada não é navmesh mundial. Jump/glide comprometidos não recebem desvio arbitrário posterior."
    ],
    "commands": [
      "query_navigation_generation",
      "set_navigation_generation",
      "inspect_npc_circulation"
    ],
    "sources": [
      "docs/npc-navigation-generation.md",
      "docs/npc-circulation.md",
      "docs/npc-air-circulation.md"
    ],
    "status": "implemented",
    "media": "npc"
  },
  {
    "id": "system-52",
    "roots": [
      "npcs",
      "world"
    ],
    "title": [
      "Population, templates & identity",
      "População, templates e identidade"
    ],
    "body": [
      "Spawning and removing actors are runtime operations separate from the document. Templates initialize components; generation references prevent old memories from being inherited by a new slot occupant.",
      "Nascer e retirar atores são operações de runtime, separadas do documento. Templates inicializam componentes, referências de geração impedem que memórias antigas sejam herdadas por um novo ocupante do slot."
    ],
    "features": [
      [
        "Full-footprint spawn support",
        "Capacity before identity consumption",
        "Configurable NPCs up to 128 in the current scope"
      ],
      [
        "Apoio integral antes do nascimento",
        "Capacidade antes de consumir identidade",
        "NPCs configuráveis até 128 no recorte atual"
      ]
    ],
    "limit": [
      "1,000 players does not prove1,000 NPCs with full sensing and planning.",
      "1.000 jogadores não prova1.000 NPCs com sensores e planejamento completos."
    ],
    "commands": [
      "spawn_actor",
      "despawn_actor",
      "inspect_population",
      "inspect_instances",
      "set_npc_capacity"
    ],
    "sources": [
      "docs/spawn-templates.md",
      "docs/instance-identity.md",
      "docs/npc-population-scale.md"
    ],
    "status": "implemented",
    "media": "npc"
  },
  {
    "id": "system-53",
    "roots": [
      "npcs",
      "world"
    ],
    "title": [
      "Generators, regions & waves",
      "Geradores, regiões e ondas"
    ],
    "body": [
      "Continuous population and finite waves share templates, quotas and identity. Seeded regions and weights select candidates; generator activation, pause and control are explicit.",
      "População contínua e ondas finitas compartilham templates, quotas e identidade. Regiões e pesos seeded escolhem candidatos; ativação, pausa e controle do gerador são explícitos."
    ],
    "features": [
      [
        "Historical wave completion",
        "Cadence/budget/live population target",
        "Failure does not consume identity or success quota"
      ],
      [
        "Conclusão histórica de onda",
        "Cadência/budget/meta viva",
        "Falha não consome identidade ou quota de sucesso"
      ]
    ],
    "limit": [
      "Spawning does not guarantee a viable route or behavior at every chosen point.",
      "Spawn não garante rota ou comportamento viável em qualquer ponto escolhido."
    ],
    "commands": [
      "control_npc_generator",
      "inspect_generators",
      "set_npc_generators",
      "query_spawn_support"
    ],
    "sources": [
      "docs/population-generators.md",
      "docs/population-waves.md",
      "docs/spawn-regions.md",
      "docs/weighted-population.md"
    ],
    "status": "implemented",
    "media": "npc"
  },
  {
    "id": "system-54",
    "roots": [
      "network",
      "combat"
    ],
    "title": [
      "Runtime for 1,000 players",
      "Runtime para 1.000 jogadores"
    ],
    "body": [
      "Identity, queues, per-actor controls, local transactions and delivery pools were scaled beyond the initial laboratory. Continuous mode does not depend on an endless replay history.",
      "Identidade, filas, controles por ator, transações locais e pools de entregas foram dimensionados além do laboratório inicial. O modo continuous não depende de um histórico de replay sem fim."
    ],
    "features": [
      [
        "1,000 bindings/actions in recorded scenarios",
        "Owner/source indices and budgets",
        "Preserved offline single player"
      ],
      [
        "1.000 vínculos e ações nos cenários registrados",
        "Índices e budgets por dono/fonte",
        "Single player offline preservado"
      ]
    ],
    "limit": [
      "Specified local/loopback load; maximum storage limits do not guarantee 60Hz for arbitrary combat.",
      "Carga especificada local/loopback; limites máximos de armazenamento não garantem 60Hz em qualquer combate."
    ],
    "commands": [
      "bind_control",
      "set_control_input",
      "inspect_controls",
      "inspect_execution"
    ],
    "sources": [
      "docs/scalable-actor-runtime.md",
      "docs/frame-admission.md",
      "docs/joint-runtime-load.md"
    ],
    "status": "implemented",
    "media": "network"
  },
  {
    "id": "system-55",
    "roots": [
      "network"
    ],
    "title": [
      "QUIC, sessions & authority",
      "QUIC, sessões e autoridade"
    ],
    "body": [
      "Real QUIC transport with TLS, scoped ownership, deduplication, monotonic ACKs and resumption. Clients send allowed operations; connection does not grant administrative world access.",
      "Transporte QUIC real com TLS, posse escopada, deduplicação, ACK monotônico e retomada. Um cliente envia operações permitidas; conexão não concede poderes administrativos sobre o mundo."
    ],
    "features": [
      [
        "Authoritative session",
        "Atomic full/delta publication",
        "1,000 connections in loopback experiments"
      ],
      [
        "Sessão autoritativa",
        "Full/delta publicado atomicamente",
        "1.000 conexões nos ensaios loopback"
      ]
    ],
    "limit": [
      "WAN, production authentication, prediction and interpolation remain separate deliveries.",
      "WAN, autenticação de produção, predição e interpolação permanecem entregas separadas."
    ],
    "commands": [
      "start_multiplayer_host",
      "connect_multiplayer",
      "reconnect_multiplayer",
      "inspect_multiplayer_host"
    ],
    "sources": [
      "docs/quic-transport.md",
      "docs/network-session-runtime.md",
      "docs/joint-network-load.md"
    ],
    "status": "implemented",
    "media": "network"
  },
  {
    "id": "system-56",
    "roots": [
      "network",
      "character"
    ],
    "title": [
      "Remote client & presentation",
      "Cliente remoto e apresentação"
    ],
    "body": [
      "Public content, poses and remote controls reach a native window/capture without replicating private decision state. Picking, aiming, abilities and camera use the same ownership contracts.",
      "Conteúdo público, poses e controles remotos chegam a uma janela/captura nativa sem replicar estado privado de decisão. Picking, mira, habilidades e câmera passam pelos mesmos contratos de posse."
    ],
    "features": [
      [
        "Identity/life-scoped remote controls",
        "Public audio/animation state",
        "Client capture through the API"
      ],
      [
        "Controle remoto por identidade/vida",
        "Som e animação públicos",
        "Captura do cliente pela API"
      ]
    ],
    "limit": [
      "No promised web client or validated internet experience; current evidence is native and loopback.",
      "Sem promessa de cliente web ou experiência de internet validada; provas atuais são nativas e loopback."
    ],
    "commands": [
      "set_multiplayer_input",
      "pick_multiplayer",
      "use_multiplayer_ability",
      "capture_multiplayer"
    ],
    "sources": [
      "docs/remote-controls.md",
      "docs/remote-presentation.md"
    ],
    "status": "implemented",
    "media": "network"
  },
  {
    "id": "system-57",
    "roots": [
      "tools"
    ],
    "title": [
      "Checkpoints & gameplay continuity",
      "Checkpoints e continuidade de gameplay"
    ],
    "body": [
      "Durable checkpoints preserve individuals, memories, routines, clocks and native physics. Loading prepares and validates a candidate before replacing the world and neutralizes external input tokens.",
      "Checkpoints duráveis preservam indivíduos, memória, rotinas, relógios e física nativa. Load prepara e valida um candidato antes de substituir o mundo e neutraliza tokens externos de entrada."
    ],
    "features": [
      [
        "CBOR and checksum",
        "Post-load replay baseline",
        "No partial publication on failure"
      ],
      [
        "CBOR e checksum",
        "Baseline para replay pós-load",
        "Erros sem publicação parcial"
      ]
    ],
    "limit": [
      "Native physics snapshots have explicit version/platform scope; they are not universally portable saves.",
      "Snapshot físico nativo tem versão/plataforma explícitas; não é save portátil universal."
    ],
    "commands": [
      "save_checkpoint",
      "load_checkpoint",
      "inspect_checkpoint"
    ],
    "sources": [
      "docs/npc-checkpoints.md"
    ],
    "status": "implemented",
    "media": "npc"
  },
  {
    "id": "system-58",
    "roots": [
      "tools",
      "network"
    ],
    "title": [
      "Canonical portable saves",
      "Save portátil canônico"
    ],
    "body": [
      "PSV1 stores Ignis primary state without serializing Rapier arenas, handles or manifolds. The adapter rebuilds physics and AABBs without advancing a tick, publishing a validated world.",
      "PSV1 guarda estado primário da Ignis sem serializar arenas, handles ou manifolds do Rapier. O adapter recompõe física e AABBs sem avançar um tick e publica o mundo validado."
    ],
    "features": [
      [
        "Explicit compatible-checkpoint migration",
        "Fresh scopes and neutral input",
        "Atomic write/replacement"
      ],
      [
        "Migração explícita de checkpoint compatível",
        "Scopes novos e input neutro",
        "Escrita/substituição atômica"
      ]
    ],
    "limit": [
      "Format portability does not prove bit-exact trajectories across backends or tested support on other systems.",
      "Portabilidade de formato não prova trajetórias bitexatas entre backends nem suporte testado em outros sistemas."
    ],
    "commands": [
      "save_portable_save",
      "load_portable_save",
      "inspect_portable_save",
      "migrate_portable_save"
    ],
    "sources": [
      "docs/portable-saves.md"
    ],
    "status": "implemented"
  },
  {
    "id": "system-59",
    "roots": [
      "world",
      "modeling"
    ],
    "title": [
      "Editable terrain & ridges",
      "Terrenos editáveis e cristas"
    ],
    "body": [
      "Fixed sampled and procedural sources use one sampler. Raising, lowering, leveling, noise and anisotropic Ridge generate native topography with editable recipes and a defined datum.",
      "Fontes fixas por amostras e fontes procedurais usam o mesmo sampler. Elevação, depressão, nivelamento, ruído e Ridge anisotrópico geram topografia nativa com receita editável e datum definido."
    ],
    "features": [
      [
        "Terrain2 with legacy reading",
        "Transactional regional publication",
        "Same published surface for visual/opt-in collision"
      ],
      [
        "Terreno2 e leitura do legado",
        "Publicação transacional por região",
        "Mesma superfície publicada para visual e colisão opt-in"
      ]
    ],
    "limit": [
      "Heightfields do not represent ceilings/caves; planar Ridge is not yet integrated into the current planet source.",
      "Heightfield não representa teto/caverna; Ridge planar ainda não integra a fonte planetária atual."
    ],
    "commands": [
      "query_terrain",
      "publish_terrain_region",
      "save_terrain",
      "load_terrain"
    ],
    "sources": [
      "docs/terrain.md",
      "docs/representative-world.md",
      "docs/terrain-collision.md"
    ],
    "status": "implemented",
    "media": "sunset"
  },
  {
    "id": "system-60",
    "roots": [
      "world",
      "modeling"
    ],
    "title": [
      "Landforms: walls, arches & volumes",
      "Landforms: paredes, arcos e volumes"
    ],
    "body": [
      "Concave profiles and parallel sections create walls, columns, ledges, peaks and real cavities. Caps and annuli preserve openings, with validation before mesh publication.",
      "Perfis côncavos e seções paralelas criam paredes, colunas, patamares, ápices e cavidades reais. Tampas e annuli preservam vãos, com validação antes de publicar malhas."
    ],
    "features": [
      [
        "f64 sources and native bakes",
        "Typed local raycasts",
        "Presets produce ordinary editable data"
      ],
      [
        "Fonte f64 e bake nativo",
        "Raycasts locais tipados",
        "Presets produzem dados editáveis comuns"
      ]
    ],
    "limit": [
      "This is not CSG/boolean union, geological erosion or universal branching caves.",
      "Não é CSG/boolean union, erosão geológica ou caverna ramificada universal."
    ],
    "commands": [
      "query_landform",
      "publish_landform",
      "save_landform",
      "load_landform"
    ],
    "sources": [
      "docs/landforms.md"
    ],
    "status": "implemented",
    "media": "sunset"
  },
  {
    "id": "system-61",
    "roots": [
      "world",
      "tools"
    ],
    "title": [
      "Global coordinates & residency",
      "Coordenadas globais e residência"
    ],
    "body": [
      "Integer-cell addresses and local offsets separate global position from resident-frame precision. Interest/dependency planning tracks region load/removal intentions without treating a seed as identity.",
      "Endereços por célula inteira e offsets locais separam posição global da precisão do frame residente. Planejamento por interesses e dependências mantém intenção de carregar/retirar regiões sem tratar seed como identidade."
    ],
    "features": [
      [
        "Canonical Position1",
        "Hysteresis and priorities",
        "Fallback retains coverage until ready"
      ],
      [
        "Position1 canônico",
        "Histerese e prioridades",
        "Fallback conserva cobertura até prontidão"
      ]
    ],
    "limit": [
      "The general planner is a read/contract layer; it is not unrestricted production multi-actor streaming.",
      "O planejador geral é leitura/contrato; não equivale a streaming multiator irrestrito em produção."
    ],
    "commands": [
      "query_world_space",
      "plan_world_regions",
      "plan_terrain_coverage"
    ],
    "sources": [
      "docs/world-space.md",
      "docs/world-residency.md",
      "docs/incremental-residency.md"
    ],
    "status": "implemented",
    "media": "sunset"
  },
  {
    "id": "system-62",
    "roots": [
      "world"
    ],
    "title": [
      "Walkable planet at Earth scale",
      "Planeta caminhável em escala terrestre"
    ],
    "body": [
      "The opt-in runtime installs/removes tiles across six faces, transports origin/frame and keeps gravity, camera and light radial. KCC walks the physical source; visual residency is separate.",
      "Runtime opt-in instala e retira tiles em seis faces, transporta origem/referencial e mantém gravidade, câmera e luz radiais. O KCC anda na fonte física; residência visual é uma camada separada."
    ],
    "features": [
      [
        "12,742km Earth-scale diameter",
        "Recorded40,030km physical circumnavigation",
        "Inspectable global pose and resident tiles"
      ],
      [
        "Diâmetro12.742km na montagem terrestre",
        "Volta física registrada de40.030km",
        "Inspeção de pose global e tiles residentes"
      ]
    ],
    "limit": [
      "One Player, without active saves/multiplayer/authoring edits. Synchronous preparation; mountain appearance is under revision.",
      "Um Player, sem save/multiplayer/edição autoral ativos. Preparação síncrona; aparência das montanhas está em revisão."
    ],
    "commands": [
      "set_planet_runtime",
      "load_planet_runtime",
      "inspect_planet_runtime"
    ],
    "sources": [
      "docs/planet-runtime.md",
      "docs/planet-appearance-audit.md"
    ],
    "status": "implemented",
    "media": "planet"
  },
  {
    "id": "system-63",
    "roots": [
      "character",
      "world"
    ],
    "title": [
      "First- & third-person exploration",
      "Passeio em primeira e terceira pessoa"
    ],
    "body": [
      "Session camera follows Player identity/life with yaw/pitch, distance and shoulder controls. Looking and movement use operations shared by native input and MCP without turning camera motion into body teleportation.",
      "Câmera de sessão acompanha a identidade/vida do Player com yaw/pitch, distância e ombro. Olhar e movimento usam operações comuns ao nativo e MCP, sem transformar câmera em teleporte do corpo."
    ],
    "features": [
      [
        "Mouse/WASD and locomotion profiles",
        "Perspective switching",
        "Explicit view inspection"
      ],
      [
        "Mouse/WASD e perfis de locomoção",
        "Alternância de perspectiva",
        "Inspeção explícita da visão"
      ]
    ],
    "limit": [
      "A complete authored human Player avatar is a separate integration; cameras do not imply FPS arm animation.",
      "Avatar humano autoral completo do Player é integração própria; câmera não implica animação FPS de braços."
    ],
    "commands": [
      "set_player_view",
      "look_player_view",
      "set_player_view_input",
      "inspect_player_view"
    ],
    "sources": [
      "docs/player-walk.md",
      "docs/player-perspectives.md"
    ],
    "status": "implemented",
    "media": "horizon"
  },
  {
    "id": "system-64",
    "roots": [
      "tools",
      "modeling"
    ],
    "title": [
      "Visual comparison & evidence-based authoring",
      "Comparação visual e autoria por evidência"
    ],
    "body": [
      "Native captures can be compared by regions/masks with RGB8, luminance SSIM and explicit coverage. Image-based authoring keeps construction inventories, hypotheses and revisions separate from results.",
      "Capturas nativas podem ser comparadas por regiões e máscaras, com RGB8, SSIM de luminância e cobertura explícita. A autoria por imagem mantém lista de construção, hipóteses e revisões distintas do resultado."
    ],
    "features": [
      [
        "Read-only comparison",
        "Landmarks and diagnostics",
        "Editable procedural sources, no photo pasted as geometry"
      ],
      [
        "Comparação read-only",
        "Landmarks e diagnósticos",
        "Fonte procedural editável, sem foto colada como geometria"
      ]
    ],
    "limit": [
      "SSIM is not a3D-fidelity percentage; study reconstructions retain artistic limitations.",
      "SSIM não é porcentagem de fidelidade3D; reconstruções de estudo ainda têm limitações artísticas."
    ],
    "commands": [
      "compare_images",
      "project_landmarks",
      "capture"
    ],
    "sources": [
      "docs/image-similarity.md",
      "docs/image-to-3d-integration.md"
    ],
    "status": "implemented",
    "media": "editor"
  },
  {
    "id": "system-65",
    "roots": [
      "npcs",
      "character"
    ],
    "title": [
      "Spatial audio & presentation events",
      "Áudio espacial e eventos de apresentação"
    ],
    "body": [
      "Presentation events feed a PCM mixer and device-free capture. CPAL output is optional; voices, distance and public state belong to presentation without feeding acoustic knowledge into sensors.",
      "Eventos de apresentação alimentam mixer PCM e captura sem dispositivo. A saída CPAL é opcional; voz, distância e estado público pertencem à apresentação, sem transferir conhecimento acústico aos sensores."
    ],
    "features": [
      [
        "24kHz mix and 64 voices in the current contract",
        "Bounded mailbox outside callback",
        "Optional output disabled by default"
      ],
      [
        "Mix 24kHz e 64 vozes no contrato atual",
        "Mailbox limitada fora do callback",
        "Reprodução opcional/desligada por padrão"
      ]
    ],
    "limit": [
      "PCM capture does not certify speakers/devices, a full music system or final artistic mixing.",
      "Captura PCM não certifica alto-falantes/dispositivo, música completa ou mixagem artística final."
    ],
    "commands": [
      "set_avatar_audio",
      "capture_npc_audio",
      "set_npc_audio_output",
      "inspect_npc_audio_output"
    ],
    "sources": [
      "docs/npc-presentation.md",
      "docs/npc-presentation-complete.md"
    ],
    "status": "implemented",
    "media": "npc"
  }
];
export const roadmap = [
  {
    "id": "M01",
    "title": [
      "Core & extensibility",
      "Núcleo e extensibilidade"
    ],
    "now": [
      "Modular Rust runtime, typed dispatcher, generation/life identity and native formats.",
      "Runtime Rust modular, dispatcher tipado, identidade por geração/vida e formatos nativos."
    ],
    "next": [
      "Public extension/language contracts; stable SDK and distribution modularity. WASM still needs porting and validation.",
      "Contratos públicos de extensão e linguagens suportadas; SDK estável e modularidade de distribuição. WASM ainda precisa de port e validação."
    ]
  },
  {
    "id": "M02",
    "title": [
      "Scenes, assets & editor",
      "Cenas, assets e editor"
    ],
    "now": [
      "Projects/scenes, configurable native editor, gizmos, selection, sculpting, modeling and procedural sources.",
      "Projetos/cenas, editor nativo configurável, gizmos, seleção, escultura, modelagem e fontes procedurais."
    ],
    "next": [
      "Expand production workflows, asset import/interchange, hierarchies and ergonomics. Modeling/preservation limits remain explicit.",
      "Expandir fluxos de produção, importação e intercâmbio de assets, hierarquias e ergonomia. Limites de modelagem e preservação continuam explícitos."
    ]
  },
  {
    "id": "M03",
    "title": [
      "Rendering, input, audio & UI",
      "Render, entrada, áudio e UI"
    ],
    "now": [
      "Procedural/dielectric/metallic materials, shadows, IBL, atmosphere, instancing, GPU occlusion, cameras and optional audio.",
      "Materiais procedurais/dielétricos/metálicos, sombras, IBL, atmosfera, instancing, oclusão GPU, câmeras e áudio opcional."
    ],
    "next": [
      "Transparency/glass in development; expand materials, visual animation and controls. IGL/LOD research needs evidence before activation. No ray-tracing or universal-parity promise.",
      "Transparência e vidro em desenvolvimento; ampliar materiais, animação visual e controles. Pesquisa IGL/LOD exige evidência antes de ativação. Sem promessa de ray tracing ou paridade universal."
    ]
  },
  {
    "id": "M04",
    "title": [
      "Physics & characters",
      "Física e personagens"
    ],
    "now": [
      "Ground KCC, slopes/steps, modular support, terrain collision, hybrid networks and per-actor controllers.",
      "KCC terrestre, rampas/degraus, apoio modular, colisão de terreno, rede híbrida e controladores por ator."
    ],
    "next": [
      "Expand bodies/queries, interaction, motor coverage and continuous-world integration. Preserve local-physics versus global-route viability.",
      "Expandir corpos/queries e interação, cobertura dos motores e integração com mundo contínuo. Preservar diferença entre consulta física local e viabilidade global."
    ]
  },
  {
    "id": "M05",
    "title": [
      "Animation & avatars",
      "Animação e avatares"
    ],
    "now": [
      "Native rigs/clips, retargeting, IK, planar root motion and human creator with parametric/facial controls.",
      "Rig/clipes nativos, retarget, IK, root motion planar e criador humano com controles paramétricos/faciais."
    ],
    "next": [
      "Final anatomy/art, correctives/expressions, clothing/equipment, Player avatars and production animation families. No automatic MetaHuman equivalence.",
      "Anatomia e arte finais, correctives/expressões, roupas/equipamentos, avatares do Player e famílias animadas de produção. Sem equivalência automática a MetaHuman."
    ]
  },
  {
    "id": "M06",
    "title": [
      "Procedural generation & topology",
      "Proceduralidade e topologia"
    ],
    "now": [
      "Constructive mesh, plant, character, terrain and landform sources with seeds and validated regeneration.",
      "Fontes construtivas de meshes, plantas, personagens, terrenos e landforms com seed e regeneração validada."
    ],
    "next": [
      "Expand composition/partial regeneration, preserve interventions and integrate world operators. Spherical Ridge and natural planet topology remain in progress.",
      "Ampliar composição e regeneração parcial, preservar intervenções e integrar operadores de mundo. Ridge esférico e topologia natural planetária continuam em evolução."
    ]
  },
  {
    "id": "M07",
    "title": [
      "Particles, water & atmosphere",
      "Partículas, água e atmosfera"
    ],
    "now": [
      "Integrated atmosphere, procedural sky/environment and botanical libraries.",
      "Atmosfera, céu/ambiente procedural e bibliotecas botânicas integradas."
    ],
    "next": [
      "VFX/particles are in requirements; materials/transparency are the foundation in development. Emitters, modules, timelines, ribbons, fields, weather, water and fluids require separate deliveries.",
      "VFX/partículas em levantamento; materiais/transparência são a fundação em desenvolvimento. Emissores, módulos, timeline, ribbons, campos, clima, água e fluidos exigem entregas próprias."
    ]
  },
  {
    "id": "M08",
    "title": [
      "Abilities & combat",
      "Habilidades e combate"
    ],
    "now": [
      "Costs/resources, casts/channeling/interruption, composed damage, buffs, areas/auras, triage, death and resurrection.",
      "Custos/recursos, cast/canalização/interrupção, dano composto, buffs, áreas/auras, triagem, morte e ressurreição."
    ],
    "next": [
      "Expand operators, tactics, melee/defense and chaining with cross-genre demonstrations; the current catalog is not complete production combat.",
      "Ampliar operadores, tática, melee/defesa e encadeamento com demonstrações de gêneros distintos; não tratar o catálogo atual como combate de produção completo."
    ]
  },
  {
    "id": "M09",
    "title": [
      "NPCs & navigation",
      "NPCs e navegação"
    ],
    "now": [
      "NPC v1 technically closed: cognition, personality, tactics/cooperation, routines/dialogue, mobility/circulation, presentation and checkpoints.",
      "NPC v1 fechado tecnicamente: cognição, personalidade, tática/cooperação, rotinas/diálogo, mobilidade/circulação, apresentação e checkpoints."
    ],
    "next": [
      "Integrated-scenario acceptance, expanded corpus/art and measured scale, continuous residency and interaction quality. Limited knowledge remains mandatory; LLM optional.",
      "Aceite do cenário conjunto, ampliar corpus/arte e escala medida, integrar residência contínua e qualidade de interação. Conhecimento limitado continua obrigatório; LLM opcional."
    ]
  },
  {
    "id": "M10",
    "title": [
      "Spawn & populations",
      "Spawn e populações"
    ],
    "now": [
      "Templates, spawn/despawn, support, generators, weighted regions, waves and configurable NPC capacity.",
      "Templates, spawn/despawn, apoio, geradores, regiões ponderadas, ondas e capacidade NPC configurável."
    ],
    "next": [
      "World-distributed persistent populations, ecology and measured scale beyond current budgets; lifecycle must preserve identity and origins.",
      "Populações persistentes distribuídas no mundo, ecologia e escala medida além dos budgets atuais; ciclo de vida deve preservar identidade e origens."
    ]
  },
  {
    "id": "M11",
    "title": [
      "Quests & questlines",
      "Quests e questlines"
    ],
    "now": [
      "",
      ""
    ],
    "next": [
      "Conditions, objectives, dependencies, branches and rewards. Individual or shared state and procedural composition; authoring, execution, inspection and persistence through one contract.",
      "Condições, objetivos, dependências, ramificações e recompensas. Estado individual ou compartilhado e composição procedural; autoria, execução, inspeção e persistência pelo mesmo contrato."
    ]
  },
  {
    "id": "M12",
    "title": [
      "Single-player, multiplayer & co-op",
      "Single-player, multiplayer e coop"
    ],
    "now": [
      "Offline single player and authoritative QUIC/TLS sessions, controls/replication/remotes;1,000 Player loopback load.",
      "Single player offline e sessões autoritativas QUIC/TLS, controles/réplica/remoto; carga de1,000 Players em loopback."
    ],
    "next": [
      "WAN, production authentication/durable ownership, prediction/interpolation, operations and sustained scale. Loopback benchmarks are not an MMO launch.",
      "WAN, autenticação e posse duráveis de produção, predição/interpolação, operação e escala sustentada. Benchmark loopback não é lançamento MMO."
    ]
  },
  {
    "id": "M13",
    "title": [
      "Open worlds & levels",
      "Mundo aberto e níveis"
    ],
    "now": [
      "Global coordinates, portable saves, region planning and opt-in walkable planet runtime.",
      "Coordenadas globais, save portátil, planejador de regiões e runtime planetário opt-in caminhável."
    ],
    "next": [
      "Asynchronous residency, spatial LOD/seams, planetary multi-actor/save integration and appearance. Current planetary mode rejects unsupported flows.",
      "Residência assíncrona e LOD espacial/costuras, integração multiator/saves no planeta e aparência. O mundo planetário atual recusa esses fluxos em vez de simular suporte."
    ]
  },
  {
    "id": "M14",
    "title": [
      "Dungeons, raids, PvP & PvE",
      "Dungeons, raids, PvP e PvE"
    ],
    "now": [
      "",
      ""
    ],
    "next": [
      "Encounters, bosses, groups, instances, checkpoints, difficulty and rewards. Factions, objectives, eligibility, scoring and victory. Reuse combat, spawn, quests and sessions; group sizes remain to be defined.",
      "Encontros, chefes, grupos, instâncias, checkpoints, dificuldade e recompensas. Facções, objetivos, elegibilidade, placar e vitória. Reutilizar combate, spawn, quests e sessões; escala de grupos a definir."
    ]
  },
  {
    "id": "M15",
    "title": [
      "Observation, tests & replay",
      "Observação, testes e replay"
    ],
    "now": [
      "Inspection/captures, typed scenarios, replay, checkpoints, portable saves, RGB8/SSIM comparison and viewport metrics.",
      "Inspeção/captura, cenários tipados, replay, checkpoints, save portátil, comparador RGB8/SSIM e métricas de viewport."
    ],
    "next": [
      "Expand diagnostics/jobs and cross-platform evidence. Performance profiles are identified scenarios; triangle counts are not FPS.",
      "Expandir diagnósticos/jobs e provas multiplataforma. Perfis de desempenho são cenários identificados; não converter contagem de triângulos em FPS."
    ]
  },
  {
    "id": "M16",
    "title": [
      "Games, export & source availability",
      "Jogos, exportação e código disponível"
    ],
    "now": [
      "Repository and commercial source license with a profit-based royalty, documentation, native examples and public website.",
      "Repositório e licença comercial de código disponível com participação no lucro, documentação, exemplos nativos e site público."
    ],
    "next": [
      "Complete3D game, cross-genre examples, independent SDK/packaging and validated multi-platform distribution. Downloadable releases need their own delivery.",
      "Jogo3D completo, exemplos de gêneros distintos, SDK/empacotamento independente e distribuição multiplataforma validada. Downloads prontos exigem release próprio."
    ]
  }
];

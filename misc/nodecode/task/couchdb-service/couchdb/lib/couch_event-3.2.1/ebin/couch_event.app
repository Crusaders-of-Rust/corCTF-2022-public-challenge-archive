{application,couch_event,
             [{description,"Event notification system for Apache CouchDB"},
              {vsn,"3.2.1"},
              {registered,[couch_event_sup,couch_event_server]},
              {applications,[kernel,stdlib,khash,couch_log,config]},
              {mod,{couch_event_app,[]}},
              {modules,[couch_event,couch_event_app,couch_event_listener,
                        couch_event_listener_mfa,couch_event_os_listener,
                        couch_event_server,couch_event_sup2]}]}.

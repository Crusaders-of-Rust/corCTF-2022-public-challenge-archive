{application,couch_peruser,
             [{description,"couch_peruser - maintains per-user databases in CouchDB"},
              {vsn,"3.2.1"},
              {registered,[couch_peruser,couch_peruser_sup]},
              {applications,[kernel,stdlib,config,couch,fabric,mem3]},
              {mod,{couch_peruser_app,[]}},
              {env,[]},
              {modules,[couch_peruser,couch_peruser_app,couch_peruser_sup]}]}.

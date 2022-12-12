module Check = struct
  module Lib = struct
    let ok ~basename =
      let filename = "tests/LIB/" ^ basename in
      Dedukti.Check.ok ~filename

    let _ = ok ~basename:"prv.dk" [Export]
  end

  module Ok = struct
    let ok ~basename =
      let filename = "tests/OK/" ^ basename in
      Dedukti.Check.ok ~filename

    let _ =
      ok ~basename:"*1.dk" [Export];
      ok ~basename:"*2.dk" [Import "tests/OK"];

      ok ~basename:"~^+ ∉a.dk" [Export];
      ok ~basename:"~^+ ∉b.dk" [Import "tests/OK"];

      ok ~basename:"dep_A.dk" [Export];
      ok ~basename:"dep_B.dk" [Import "tests/OK"];
      ok ~basename:"dep C.dk" [Export];
      ok ~basename:"dep D.dk" [Import "tests/OK"];

      ok ~basename:"SR_sat_2.dk" [Type_lhs];
      ok ~basename:"SR_sat_bv1.dk" [Type_lhs];
      ok ~basename:"SR_sat_bv2.dk" [Type_lhs];
      ok ~basename:"SR_sat_eq1.dk" [Type_lhs];
      ok ~basename:"SR_sat_eq2.dk" [Type_lhs];

      ok ~basename:"SR_OK_1.dk" [Sr_check 1];
      ok ~basename:"SR_OK_2.dk" [Sr_check 2];
      ok ~basename:"SR_OK_3.dk" [Sr_check 3];
      ok ~basename:"SR_OK_4.dk" [Sr_check 4];

      ok ~basename:"brackets0.dk" [];
      ok ~basename:"brackets0b.dk" [];
      ok ~basename:"brackets1.dk" [];
      ok ~basename:"brackets2.dk" [];
      ok ~basename:"brackets3.dk" [];
      ok ~basename:"brackets4.dk" [];
      ok ~basename:"brackets5.dk" [];

      ok ~basename:"cstr_ignored1.dk" [];
      ok ~basename:"cstr_ignored2.dk" [];
      ok ~basename:"cstr_ignored3.dk" [];

      ok ~basename:"arities.dk" [];
      ok ~basename:"arities2.dk" [];

      ok ~basename:"first_order_cstr1.dk" [];
      ok ~basename:"firstOrder_v2.dk" [];

      ok ~basename:"guard1.dk" [];
      ok ~basename:"guard2.dk" [];
      ok ~basename:"guard3.dk" [];

      ok ~basename:"higher_order_cstr1.dk" [];
      ok ~basename:"higher_order_cstr2.dk" [];
      ok ~basename:"higher_order_cstr3.dk" [];

      ok ~basename:"ho_bug1.dk" [];
      ok ~regression:true ~basename:"ho_bug2.dk" [];

      ok ~regression:true ~basename:"miller1.dk" [];
      ok ~regression:true ~basename:"miller2.dk" [];

      ok ~basename:"nonlinearity.dk" [];

      ok ~regression:true ~basename:"nsteps1.dk" [];
      ok ~regression:true ~basename:"nsteps2.dk" [];
      ok ~regression:true ~basename:"nsteps3.dk" [];

      ok ~basename:"rule_name2.dk" [];

      ok ~basename:"self_dep.dk" [];
      ok ~basename:"self_dep2.dk" [];
      ok ~basename:"self_dep3.dk" [];

      ok ~regression:true ~basename:"special_idents1.dk" [Export];
      ok ~basename:"special_idents2.dk" [Import "tests/OK"];
      ok ~basename:"special_idents3.dk" [];

      ok ~basename:"type_annot_cstr.dk" [];
      ok ~basename:"type_annot_cstr2.dk" [];
      ok ~basename:"type_annot_cstr3.dk" [];

      ok ~basename:"typable_lhs.dk" [];
      ok ~basename:"typable_lhs2.dk" [];

      ok ~basename:"underscore1.dk" [];
      ok ~basename:"underscore2.dk" [];
      ok ~basename:"underscore3.dk" [];
      ok ~basename:"underscore4.dk" [];
      ok ~basename:"underscore5.dk" [];
      ok ~basename:"underscore6.dk" [];
      ok ~basename:"underscore7.dk" [];

      ok ~basename:"domainfree.dk" [];
      ok ~basename:"pattern_parentheses_issue259.dk" [];
      ok ~basename:"firstOrder.dk" [];
      ok ~basename:"recursive.dk" [];
      ok ~regression:true ~basename:"sharing.dk" [];
      ok ~basename:"def.dk" [Export];
      ok ~basename:"require.dk" [Import "tests/OK"; Standard];
      ok ~regression:true ~basename:"inferingKindForType.dk" [];
      ok ~basename:"ho_match.dk" [];
      ok ~basename:"tptp.dk" [];
      ok ~regression:true ~basename:"inferingKindForArrowWithCodomainType.dk" [];
      ok ~basename:"doubleneg.dk" [];
      ok ~basename:"WIP.dk" [];
      ok ~basename:"ho_nonlinearity.dk" [];
      ok ~basename:"subst.dk" [];
      ok ~basename:"rule_name.dk" [];
      ok ~basename:"let_syntax.dk" [];
      ok ~basename:"type_annot_readme.dk" [];
      ok ~basename:"injective_smb.dk" [];
      ok ~basename:"hott.dk" [];
      ok ~basename:"nested_comments.dk" [];
      ok ~regression:true ~basename:"fixpoints.dk" [];
      ok ~basename:"type_annot.dk" [];
      ok ~basename:"rule_order.dk" [];
      ok ~regression:true ~basename:"nested_miller_pattern.dk" [];
      ok ~regression:true ~basename:"nsteps4.dk" [];
      ok ~basename:"dotpat.dk" [];
      ok ~basename:"type_rewrite.dk" [];
      ok ~basename:"pragma.dk" []

    module Acu = struct
      let ok ~basename =
        let filename = "tests/OK/acu/" ^ basename in
        Dedukti.Check.ok ~filename

      let _ =
        ok ~basename:"cc.dk" [];
        ok ~basename:"easy_ac.dk" [];
        ok ~basename:"extra_rules.dk" [];
        ok ~basename:"extra_rules2.dk" [];
        ok ~basename:"extra_rules3.dk" [];
        ok ~basename:"extra_rules4.dk" [];
        ok ~basename:"min_ac.dk" []
    end

    module Assert = struct
      let ok ~basename =
        let filename = "tests/OK/assert/" ^ basename in
        Dedukti.Check.ok ~filename

      let _ =
        ok ~regression:true ~basename:"conv_command.dk" [];
        ok ~regression:true ~basename:"type_command.dk" []
    end

    module Check = struct
      let ok ~basename =
        let filename = "tests/OK/check/" ^ basename in
        Dedukti.Check.ok ~filename

      let _ =
        ok ~regression:true ~basename:"conv_command2.dk" [];
        ok ~regression:true ~basename:"type_command2.dk" [];
        ok ~basename:"typed_lambda.dk" []
    end

    module Eta = struct
      let ok ~basename =
        let filename = "tests/OK/eta/" ^ basename in
        Dedukti.Check.ok ~filename

      let _ =
        ok ~basename:"eta_0b.dk" [];
        ok ~basename:"eta_0.dk" [Eta];
        ok ~basename:"eta_1.dk" [Eta];
        ok ~basename:"eta_2b.dk" [];
        ok ~basename:"eta_2.dk" [Eta];
        ok ~basename:"eta_arity.dk" [Eta]
    end
  end

  module Ko = struct
    let ko ~error ~basename =
      let filename = "tests/KO/" ^ basename in
      Dedukti.Check.ko ~error ~filename

    let _ =
      ko ~error:(`Code 506) ~basename:"arity.dk" [];
      ko ~error:(`Code 101) ~basename:"arrowCodomainType.dk" [];
      ko ~error:(`Code 101) ~basename:"arrowDomainType2.dk" [];
      ko ~error:(`Code 101) ~basename:"arrowDomainType.dk" [];
      ko ~error:(`Code 704) ~basename:"assert_conv_command.dk" [];
      ko ~error:(`Code 704) ~basename:"assert_not_conv_command.dk" [];
      ko ~error:(`Code 704) ~basename:"assert_not_type_command.dk" [];
      ko ~error:(`Code 704) ~basename:"assert_type_command.dk" [];
      ko ~error:(`Code 705) ~basename:"betaLHS.dk" [];
      ko ~error:(`Code 108) ~basename:"binded_cstr_fail1.dk" [Type_lhs];
      ko ~error:(`Code 109) ~basename:"brackets1.dk" [];
      ko ~error:(`Code 202) ~basename:"brackets2.dk" [];
      ko ~error:(`Code 202) ~basename:"brackets3.dk" [];
      ko ~error:(`Code 703) ~basename:"brackets4.dk" [];
      ko ~error:(`Code 401) ~basename:"brackets5.dk" [];
      ko ~error:(`Code 703) ~basename:"brackets6.dk" [];
      ko ~error:(`Code 107) ~basename:"cannot_infer_type_of_pattern_9.dk" [];
      ko ~error:(`Code 101) ~basename:"church.dk" [];
      ko ~error:(`Code 108) ~basename:"constraint_unsat.dk" [Type_lhs];
      ko ~error:(`Code 704) ~basename:"convertibility_check_types.dk" [];
      ko ~error:(`Code 108) ~basename:"cstr_fail1.dk" [Type_lhs];
      ko ~error:(`Code 108) ~basename:"cstr_fail2.dk" [Type_lhs];
      ko ~error:(`Code 101) ~basename:"def.dk" [];
      ko ~error:(`Code 106) ~basename:"domain_free_lambda_8.dk" [];
      ko ~error:(`Code 101) ~basename:"EtaInConstraints_183.dk" [];
      ko ~error:(`Code 101) ~basename:"EtaInConstraints_MoreEta_183.dk" [];
      ko ~error:(`Code 401) ~basename:"guard1.dk" [];
      ko ~error:(`Code 401) ~basename:"guard2.dk" [];
      ko ~error:(`Code 706) ~basename:"guardedApplied.dk" [];
      ko ~error:(`Code 101) ~basename:"illTypedPi.dk" [];
      ko ~error:(`Code 105) ~basename:"inexpected_kind_7.dk" [];
      ko ~error:(`Code 108) ~basename:"kind_cstr_ignored1.dk" [Type_lhs];
      ko ~error:(`Code 108) ~basename:"kind_cstr_ignored2.dk" [Type_lhs];
      ko ~error:(`Code 306) ~basename:"lambdas_type_in_type.dk" [];
      ko ~error:(`Code 701) ~basename:"lexing_id.dk" [];
      ko ~error:(`Code 306) ~basename:"nested_comments_1.dk" [];
      ko ~error:(`Code 701) ~basename:"nested_comments_2.dk" [];
      ko ~error:(`Code 101) ~basename:"noninjectivity.dk" [];
      ko ~error:(`Code 507) ~basename:"nonleftlinear.dk" [Left_linear];
      ko ~error:(`Code 701) ~basename:"nsteps1.dk" [];
      ko ~error:(`Code 702) ~basename:"parsing_eof.dk" [];
      ko ~error:(`Code 104) ~basename:"product_expected_6.dk" [];
      ko ~error:(`Code 403) ~basename:"prv_fail_1.dk" [Import "tests/LIB"];
      ko ~error:(`Code 403) ~basename:"prv_fail_2.dk" [Import "tests/LIB"];
      ko ~error:(`Code 107) ~basename:"rule_var.dk" [];
      ko ~error:(`Code 900) ~basename:"scoping_ext.dk" [];
      ko ~error:(`Code 101) ~basename:"self_dep2.dk" [];
      ko ~error:(`Code 101) ~basename:"self_dep.dk" [];
      ko ~error:(`Code 103) ~basename:"sort_expected_5.dk" [];
      ko ~error:(`Code 701) ~basename:"special_idents.dk" [];
      ko ~error:(`Code 108) ~basename:"SR_unsat_1.dk" [Type_lhs];
      ko ~error:(`Code 108) ~basename:"SR_unsat_2.dk" [Type_lhs];
      ko ~error:(`Code 108) ~basename:"SR_unsat_a1.dk" [Type_lhs];
      ko ~error:(`Code 108) ~basename:"SR_unsat_a2_2.dk" [Type_lhs];
      ko ~error:(`Code 108) ~basename:"SR_unsat_a2.dk" [Type_lhs];
      ko ~error:(`Code 108) ~basename:"SR_unsat_b1.dk" [Type_lhs];
      ko ~error:(`Code 108) ~basename:"SR_unsat_b2.dk" [Type_lhs];
      ko ~error:(`Code 108) ~basename:"SR_unsat_b3.dk" [Type_lhs];
      ko ~error:(`Code 108) ~basename:"SR_unsat_c1.dk" [Type_lhs];
      ko ~error:(`Code 108) ~basename:"SR_unsat_c2.dk" [Type_lhs];
      ko ~error:(`Code 108) ~basename:"SR_unsat.dk" [Type_lhs];
      ko ~error:(`Code 108) ~basename:"SR_unsat_e1.dk" [Type_lhs];
      ko ~error:(`Code 108) ~basename:"SR_unsat_e2.dk" [Type_lhs];
      ko ~error:(`Code 108) ~basename:"SR_unsat_f1.dk" [Type_lhs];
      ko ~error:(`Code 306) ~basename:"require.dk" [Standard];
      ko ~error:(`Code 306) ~basename:"symbol_not_found_31.dk" [];
      ko ~error:(`Code 208) ~basename:"type_annot1.dk" [];
      ko ~error:(`Code 208) ~basename:"type_annot2.dk" [];
      ko ~error:(`Code 101) ~basename:"typeArrowType.dk" [];
      ko ~error:(`Code 101) ~basename:"typing_abstraction.dk" [];
      ko ~error:(`Code 104) ~basename:"typing_omega.dk" [];
      ko ~error:(`Code 101) ~basename:"typing_pi.dk" [];
      ko ~error:(`Code 103) ~basename:"typing_sort.dk" [];
      ko ~error:(`Code 101) ~basename:"unsound.dk" [];
      ko ~error:(`Code 101) ~basename:"untypable_lhs2.dk" [];
      ko ~error:(`Code 101) ~basename:"untypable_lhs.dk" [];
      ko ~error:(`Code 306) ~basename:"pragma.dk" [];
      ko ~error:(`Code 704) ~basename:"opacity.dk" [];
      ko ~error:(`Code 704) ~basename:"opacity2.dk" []

    module Acu = struct
      let ko ~error ~basename =
        let filename = "tests/KO/acu/" ^ basename in
        Dedukti.Check.ko ~error ~filename

      let _ = ko ~error:(`Code 302) ~basename:"fail_ac.dk" []
    end

    module Eta = struct
      let ko ~error ~basename =
        let filename = "tests/KO/eta/" ^ basename in
        Dedukti.Check.ko ~error ~filename

      let _ =
        ko ~error:(`Code 704) ~basename:"eta_0.dk" [];
        ko ~error:(`Code 704) ~basename:"eta_1b.dk" [];
        ko ~error:(`Code 101) ~basename:"eta_1.dk" [];
        ko ~error:(`Code 704) ~basename:"eta_2.dk" [];
        ko ~error:(`Code 506) ~basename:"eta_arity.dk" []
    end
  end
end

let _ =
  Dedukti.Meta.run ~filename:"tests/meta/simple.dk" [];
  Dedukti.Meta.run ~filename:"tests/meta/simple.dk" [No_meta];
  Dedukti.Meta.run ~filename:"tests/meta/beta.dk" [];
  Dedukti.Meta.run ~filename:"tests/meta/beta.dk" [No_beta];
  Dedukti.Meta.run ~filename:"tests/meta/beta.dk" [No_beta; No_meta];
  Dedukti.Meta.run ~dep:["tests/meta/simple_2.dk"]
    ~filename:"tests/meta/simple_2.dk"
    [Meta "tests/meta_files/meta.dk"];
  Dedukti.Meta.run ~dep:["tests/meta/simple_2.dk"]
    ~filename:"tests/meta/simple_2.dk"
    [Meta "tests/meta_files/meta.dk"; Meta "tests/meta_files/meta2.dk"];
  Dedukti.Meta.run
    ~dep:["tests/meta/rewrite_prod.dk"]
    ~check_output:false ~filename:"tests/meta/rewrite_prod.dk"
    [Meta "tests/meta_files/prod_meta.dk"; Quoting `Prod; No_unquoting];
  Dedukti.Meta.run
    ~dep:["tests/meta/rewrite_prod.dk"]
    ~filename:"tests/meta/rewrite_prod.dk"
    [Meta "tests/meta_files/prod_meta.dk"; Quoting `Prod];
  Dedukti.Pretty.run ~filename:"tests/OK/hott.dk" [];
  Dedukti.Universo.run ~filename:"tests/universo/simple_ok.dk"
    [
      Config "tests/universo/config/universo_cfg.dk";
      Theory "tests/universo/theory/cts.dk";
      Import "tests/universo/theory";
      Output_directory "tests/universo/output";
    ];
  (* TODO: fix this one *)
  (* Dedukti.Universo.run ~filename:"tests/universo/simple_ok.dk"
   *   [
   *     Config "tests/universo/config/universo_cfg.dk";
   *     Theory "tests/universo/theory/cts.dk";
   *     Import "tests/universo/theory";
   *     Output_directory "tests/universo/output";
   *     Simplify "tests/universo/simplified_output";
   *   ]; *)
  (* TODO: fix this one too *)
  (* Dedukti.Universo.run ~fails:true ~filename:"tests/universo/simple_ko.dk"
   *   [
   *     Config "tests/universo/config/universo_cfg.dk";
   *     Theory "tests/universo/theory/cts.dk";
   *     Import "tests/universo/theory";
   *     Output_directory "tests/universo/output";
   *   ]; *)
  Test.run ()

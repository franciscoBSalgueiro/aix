#include "aixchess_functions.hpp"

namespace duckdb {

namespace {

struct MatchesFenBindData : public FunctionData {
	explicit MatchesFenBindData(Fen fen_p, bool is_null) : fen(std::move(fen_p)), is_null(is_null) {
	}

	Fen fen;
	bool is_null;

	unique_ptr<FunctionData> Copy() const override {
		return make_uniq<MatchesFenBindData>(fen, is_null);
	}

	bool Equals(const FunctionData &other_p) const override {
		auto &other = other_p.Cast<MatchesFenBindData>();
		return fen.white == other.fen.white && fen.black == other.fen.black && fen.king == other.fen.king &&
		       fen.queen == other.fen.queen && fen.rook == other.fen.rook && fen.bishop == other.fen.bishop &&
		       fen.knight == other.fen.knight && fen.pawn == other.fen.pawn &&
		       fen.white_to_move == other.fen.white_to_move && fen.castling_rights == other.fen.castling_rights &&
		       fen.ep_square == other.fen.ep_square;
	}
};

static unique_ptr<FunctionData> MatchesFenBindFunction(ClientContext &context, ScalarFunction &bound_function,
                                                       vector<unique_ptr<Expression>> &arguments) {
	auto &fen_arg = arguments[1];
	if (fen_arg->HasParameter()) {
		throw ParameterNotResolvedException();
	}
	if (!fen_arg->IsFoldable()) {
		throw InvalidInputException(*fen_arg, "fen must be a constant");
	}
	Value options_str = ExpressionExecutor::EvaluateScalar(context, *fen_arg);
	auto fen_string = options_str.GetValue<string>();
	Fen fen;
	bool is_null = options_str.IsNull();
	if (!is_null) {
		auto fen_result = Fen::parse(fen_string);
		if (fen_result.is_err()) {
			throw InvalidInputException(*fen_arg, "failed to parse fen");
		}
		fen = *std::move(fen_result).ok();
	}
	return make_uniq<MatchesFenBindData>(fen, is_null);
}

inline void MatchesFen(DataChunk &args, ExpressionState &state, Vector &result) {
	auto &func_expr = state.expr.Cast<BoundFunctionExpression>();
	auto &info = func_expr.bind_info->Cast<MatchesFenBindData>();

	if (info.is_null) {
		result.SetVectorType(VectorType::CONSTANT_VECTOR);
		ConstantVector::SetNull(result, true);
		return;
	}

	auto &game_vector = args.data[0];
	auto count = args.size();

	auto fen = info.fen;

	UnaryExecutor::Execute<string_t, bool>(game_vector, result, count, [&](string_t game) {
		diplomat::span<const uint8_t> data = {const_data_ptr_cast(game.GetData()), game.GetSize()};
		return UnwrapDecoded<bool>(fen.matches_fen(data), "matches_fen");
	});
}

inline void MatchesFenPly(DataChunk &args, ExpressionState &state, Vector &result) {
	auto &func_expr = state.expr.Cast<BoundFunctionExpression>();
	auto &info = func_expr.bind_info->Cast<MatchesFenBindData>();

	if (info.is_null) {
		result.SetVectorType(VectorType::CONSTANT_VECTOR);
		ConstantVector::SetNull(result, true);
		return;
	}

	auto fen = info.fen;

	UnaryExecutor::ExecuteWithNulls<string_t, uint16_t>(args.data[0], result, args.size(),
	                                                   [&](string_t game, ValidityMask &mask, idx_t idx) {
		                                                   diplomat::span<const uint8_t> data = {const_data_ptr_cast(game.GetData()), game.GetSize()};
		                                                   auto ply_result = fen.matches_fen_ply(data);
		                                                   auto ply_opt = UnwrapOptionalDecoded<uint16_t>(std::move(ply_result), "matches_fen_ply");

		                                                   if (!ply_opt.has_value()) {
			                                                   mask.SetInvalid(idx);
			                                                   return uint16_t(0);
		                                                   }

		                                                   return *ply_opt;
	                                                   });
}

inline void MatchesFenFromFen(DataChunk &args, ExpressionState &state, Vector &result) {
	auto &func_expr = state.expr.Cast<BoundFunctionExpression>();
	auto &info = func_expr.bind_info->Cast<MatchesFenBindData>();

	if (info.is_null) {
		result.SetVectorType(VectorType::CONSTANT_VECTOR);
		ConstantVector::SetNull(result, true);
		return;
	}

	auto fen = info.fen;

	auto count = args.size();
	result.SetVectorType(VectorType::FLAT_VECTOR);
	auto &validity = FlatVector::Validity(result);

	UnifiedReader<string_t> game_reader;
	UnifiedReader<string_t> fen_reader;
	game_reader.Init(args.data[0], count);
	fen_reader.Init(args.data[2], count);

	auto out = FlatVector::GetData<bool>(result);
	for (idx_t i = 0; i < count; i++) {
		if (game_reader.IsNull(i)) {
			validity.SetInvalid(i);
			continue;
		}

		auto game = game_reader.Get(i);
		diplomat::span<const uint8_t> data = {const_data_ptr_cast(game.GetData()), game.GetSize()};

		out[i] = fen_reader.IsNull(i)
		             ? UnwrapDecoded<bool>(fen.matches_fen(data), "matches_fen")
		             : UnwrapDecoded<bool>(
		                   fen.matches_fen_from_fen(
		                       data,
		                       std::string_view(fen_reader.Get(i).GetData(), fen_reader.Get(i).GetSize())),
		                   "matches_fen");
	}
}

inline void MatchesFenPlyFromFen(DataChunk &args, ExpressionState &state, Vector &result) {
	auto &func_expr = state.expr.Cast<BoundFunctionExpression>();
	auto &info = func_expr.bind_info->Cast<MatchesFenBindData>();

	if (info.is_null) {
		result.SetVectorType(VectorType::CONSTANT_VECTOR);
		ConstantVector::SetNull(result, true);
		return;
	}

	auto fen = info.fen;

	auto count = args.size();
	result.SetVectorType(VectorType::FLAT_VECTOR);
	auto &validity = FlatVector::Validity(result);

	UnifiedReader<string_t> game_reader;
	UnifiedReader<string_t> fen_reader;
	game_reader.Init(args.data[0], count);
	fen_reader.Init(args.data[2], count);

	auto out = FlatVector::GetData<uint16_t>(result);
	for (idx_t i = 0; i < count; i++) {
		if (game_reader.IsNull(i)) {
			validity.SetInvalid(i);
			continue;
		}

		auto game = game_reader.Get(i);
		diplomat::span<const uint8_t> data = {const_data_ptr_cast(game.GetData()), game.GetSize()};

		auto ply_result = fen_reader.IsNull(i)
		                    ? fen.matches_fen_ply(data)
		                    : fen.matches_fen_ply_from_fen(
		                          data,
		                          std::string_view(fen_reader.Get(i).GetData(), fen_reader.Get(i).GetSize()));
		auto ply_opt = UnwrapOptionalDecoded<uint16_t>(std::move(ply_result), "matches_fen_ply");

		if (!ply_opt.has_value()) {
			validity.SetInvalid(i);
			continue;
		}

		out[i] = *ply_opt;
	}
}

} // namespace

void Register_MatchesFen(ExtensionLoader &loader) {
	auto matches_fen_function = ScalarFunction("matches_fen", {LogicalType::BLOB, LogicalType::VARCHAR},
	                                           LogicalType::BOOLEAN, MatchesFen, MatchesFenBindFunction);
	loader.RegisterFunction(matches_fen_function);

	auto matches_fen_from_fen_function =
	    ScalarFunction("matches_fen", {LogicalType::BLOB, LogicalType::VARCHAR, LogicalType::VARCHAR},
	                   LogicalType::BOOLEAN, MatchesFenFromFen, MatchesFenBindFunction);
	loader.RegisterFunction(matches_fen_from_fen_function);

	auto matches_fen_ply_function = ScalarFunction("matches_fen_ply", {LogicalType::BLOB, LogicalType::VARCHAR},
	                                               LogicalType::USMALLINT, MatchesFenPly,
	                                               MatchesFenBindFunction);
	loader.RegisterFunction(matches_fen_ply_function);

	auto matches_fen_ply_from_fen_function =
	    ScalarFunction("matches_fen_ply", {LogicalType::BLOB, LogicalType::VARCHAR, LogicalType::VARCHAR},
	                   LogicalType::USMALLINT, MatchesFenPlyFromFen, MatchesFenBindFunction);
	loader.RegisterFunction(matches_fen_ply_from_fen_function);
}

} // namespace duckdb

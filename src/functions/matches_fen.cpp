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

// NULL means absent; '*' means the first matching position ends the game.
void NextMoveAtFen(DataChunk &args, ExpressionState &state, Vector &result) {
    auto &expr = state.expr.Cast<BoundFunctionExpression>();
    auto &info = expr.bind_info->Cast<MatchesFenBindData>();
    if (info.is_null) {
        result.SetVectorType(VectorType::CONSTANT_VECTOR);
        ConstantVector::SetNull(result, true);
        return;
    }
    auto fen = info.fen;
    UnifiedReader<string_t> games, starts;
    games.Init(args.data[0], args.size());
    const bool has_start = args.ColumnCount() == 3;
    if (has_start) starts.Init(args.data[2], args.size());
    result.SetVectorType(VectorType::FLAT_VECTOR);
    auto &validity = FlatVector::Validity(result);
    auto out = FlatVector::GetData<string_t>(result);
    for (idx_t i = 0; i < args.size(); i++) {
        if (games.IsNull(i)) { validity.SetInvalid(i); continue; }
        auto game = games.Get(i);
        diplomat::span<const uint8_t> data = {const_data_ptr_cast(game.GetData()), game.GetSize()};
        std::optional<uint32_t> next;
        if (has_start && !starts.IsNull(i)) {
            auto start = starts.Get(i);
            next = UnwrapOptionalDecoded<uint32_t>(fen.next_move_at_fen_from_fen(
                data, std::string_view(start.GetData(), start.GetSize())), "next_move_at_fen");
        } else {
            next = UnwrapOptionalDecoded<uint32_t>(fen.next_move_at_fen(data), "next_move_at_fen");
        }
        if (!next.has_value()) { validity.SetInvalid(i); continue; }
        if (*next == 0) { out[i] = StringVector::AddString(result, "*"); continue; }
        const auto code = *next - 1;
        const auto from = code & 63, to = (code >> 6) & 63, promotion = code >> 12;
        const char uci[] = {char('a' + from % 8), char('1' + from / 8),
                            char('a' + to % 8), char('1' + to / 8), char(promotion)};
        out[i] = StringVector::AddString(result, uci, promotion ? 5 : 4);
    }
}

} // namespace

void Register_MatchesFen(ExtensionLoader &loader) {
    for (bool custom_start : {false, true}) {
        vector<LogicalType> types = {LogicalType::BLOB, LogicalType::VARCHAR};
        if (custom_start) types.push_back(LogicalType::VARCHAR);
        auto function = ScalarFunction("next_move_at_fen", types, LogicalType::VARCHAR,
                                       NextMoveAtFen, MatchesFenBindFunction);
        function.null_handling = FunctionNullHandling::SPECIAL_HANDLING;
        loader.RegisterFunction(function);
    }
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

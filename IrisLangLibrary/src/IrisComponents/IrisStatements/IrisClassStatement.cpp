#include "IrisComponents/IrisStatements/IrisClassStatement.h"
#include "IrisComponents/IrisExpressions/IrisExpression.h"
#include "IrisUnil/IrisBlock.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisCompiler.h"
#include "IrisInstructorMaker.h"
#include "IrisFatalErrorHandler.h"
#include "IrisValidator/IrisStatementValidateVisitor.h"
#include "IrisValidator/IrisExpressionValidateVisitor.h"

IrisClassStatement::IrisClassStatement(IrisIdentifier* pClasssName, IrisExpression* pSuperClassName, IrisList<IrisExpression*>* pModules, IrisList<IrisExpression*>* pInterfaces, IrisBlock* pBlock) : m_pClassName(pClasssName), m_pSuperClassName(pSuperClassName), m_pModules(pModules), m_pInterfaces(pInterfaces), m_pBlock(pBlock)
{
}


bool IrisClassStatement::Generate()
{
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	pCompiler->SetLineNumber(m_nLineNumber);

	pMaker->push_env();
	pMaker->cre_cenv(0);

	pCompiler->IncreamDefineIndex();

	pMaker->def_cls(pCompiler->GetIdentifierIndex(m_pClassName->GetIdentifierString(), pCompiler->GetCurrentFileIndex()), pCompiler->GetDefineIndex());

	// SuperClass
	if (m_pSuperClassName) {
		if (!m_pSuperClassName->Generate()) {
			return false;
		}
		pMaker->add_ext();
	}

	// Modules
	if (m_pModules) {
		if(!m_pModules->Ergodic(
			[&](IrisExpression* pExpression) -> bool {
			if (!pExpression->Generate()) {
				return false;
			}
			pMaker->add_mld();
			return true;
		}
		))
			return false;
	}

	// Interfaces
	if (m_pInterfaces) {
		if(!m_pInterfaces->Ergodic(
			[&](IrisExpression* pExpression) -> bool {
			if (!pExpression->Generate()) {
				return false;
			}
			pMaker->add_inf();
			return true;
		}
			))
			return false;
	}

	// Block
	if (m_pBlock) {
		m_pBlock->Generate();
	}
	else {
		pCompiler->IncreamDefineIndex();
		pMaker->blk_def(pCompiler->GetDefineIndex());
		pMaker->end_def(pCompiler->GetDefineIndex());
		pCompiler->DecreamDefineIndex();
	}
	pMaker->end_def(pCompiler->GetDefineIndex());
	pCompiler->DecreamDefineIndex();
	pMaker->pop_env();
	return true;
}

IrisClassStatement::~IrisClassStatement()
{
	if (m_pClassName)
		delete m_pClassName;
	if (m_pSuperClassName)
		delete m_pSuperClassName;
	if (m_pModules) {
		m_pModules->Ergodic([](IrisExpression*& x) -> bool { delete x; x = nullptr; return true; });
		m_pModules->Clear();
	}
	if (m_pInterfaces) {
		m_pInterfaces->Ergodic([](IrisExpression*& x) -> bool { delete x; x = nullptr; return true; });
		m_pInterfaces->Clear();
		delete m_pInterfaces;
	}
	if (m_pBlock)
		delete m_pBlock;
}

bool IrisClassStatement::Validate()
{
	auto pCompiler = IrisCompiler::CurrentCompiler();

	IrisExpressionValidateVisitor ievvExpressionVisitor;
	IrisStatementValidateVisitor isvvStatementVisitor;

	if (!pCompiler->UpperStackEmpty() && pCompiler->GetTopUpperType() != IrisCompiler::UpperType::ModuleBlock) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "class of " + m_pClassName->GetIdentifierString() + " must be defined in Main environment or Module body.");
		return false;
	}

	if (m_pClassName->GetType() != IrisIdentifierType::Constance) {
		IrisFatalErrorHandler::CurrentFatalHandler()->ShowFatalErrorMessage(IrisFatalErrorHandler::FatalErrorType::IdenfierTypeIrregular, m_nLineNumber, pCompiler->GetCurrentFileIndex(), "Identifier of " + m_pClassName->GetIdentifierString() + " is not a CONSTANCE.");
		return false;
	}

	if (m_pSuperClassName && !m_pSuperClassName->Accept(&ievvExpressionVisitor)) {
		return false;
	}

	if (m_pModules && !m_pModules->Ergodic([&](IrisExpression*& pExpression) -> bool {
		if (!pExpression->Accept(&ievvExpressionVisitor)) {
			return false;
		}
		return true;
	})) {
		return false;
	}

	if (m_pInterfaces && !m_pModules->Ergodic([&](IrisExpression*& pExpression) -> bool {
		if (!pExpression->Accept(&ievvExpressionVisitor)) {
			return false;
		}
		return true;
	})) {
		return false;
	}

	if (m_pBlock) {
		pCompiler->PushUpperType(IrisCompiler::UpperType::ClassBlock);

		if (!m_pBlock->Accept(&isvvStatementVisitor)) {
			return false;
		}

		pCompiler->PopUpperType();
	}

	return true;
}
